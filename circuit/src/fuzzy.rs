use std::marker::PhantomData;

use crate::poseidon_circuit::*;
use halo2_base::halo2_proofs::circuit::{Region, Value};
use halo2_base::halo2_proofs::halo2curves::bn256::Fr;
use halo2_base::halo2_proofs::halo2curves::FieldExt;
use halo2_base::halo2_proofs::plonk::ConstraintSystem;
use halo2_base::halo2_proofs::{circuit::Layouter, plonk::Error};
use halo2_base::{
    gates::{flex_gate::FlexGateConfig, range::RangeConfig, GateInstructions, RangeInstructions},
    AssignedValue, Context,
};
use halo2_base::{ContextParams, QuantumCell};
use itertools::Itertools;

#[derive(Debug, Clone)]
pub struct FuzzyCommitmentResult<'a> {
    pub(crate) assigned_commitment: Vec<AssignedValue<'a, Fr>>,
    pub(crate) assigned_feature_hash: AssignedValue<'a, Fr>,
    pub(crate) assigned_word: Vec<AssignedValue<'a, Fr>>,
    pub(crate) word_value: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct FuzzyCommitmentConfig {
    range_config: RangeConfig<Fr>,
    error_threshold: u64,
    pub(crate) word_size: usize,
    _f: PhantomData<Fr>,
}

impl FuzzyCommitmentConfig {
    pub fn configure(
        meta: &mut ConstraintSystem<Fr>,
        range_config: RangeConfig<Fr>,
        error_threshold: u64,
        word_size: usize,
    ) -> Self {
        Self {
            range_config,
            error_threshold,
            word_size,
            _f: PhantomData,
        }
    }

    pub fn recover_and_hash<'v: 'a, 'a>(
        &self,
        ctx: &mut Context<'v, Fr>,
        poseidon: &'a PoseidonChipBn254_8_58<'v, Fr>,
        features: &[u8],
        errors: &[u8],
        commitment: &[u8],
    ) -> Result<FuzzyCommitmentResult<'a>, Error> {
        // TODO: should be no difference
        // assert_eq!(features.len(), self.word_size); 658
        // assert_eq!(errors.len(), self.word_size); 274
        // assert_eq!(commitment.len(), self.word_size); 274
        let range = self.range();
        let gate = self.gate();
        let assigned_features = features
            .into_iter()
            .map(|val| gate.load_witness(ctx, Value::known(Fr::from(*val as u64))))
            .collect_vec();
        println!("assigned_features: {:?}", assigned_features);
        let assigned_errors = errors
            .into_iter()
            .map(|val| gate.load_witness(ctx, Value::known(Fr::from(*val as u64))))
            .collect_vec();
        println!("assigned_errors: {:?}", assigned_errors);
        let assigned_commitment = commitment
            .into_iter()
            .map(|val| gate.load_witness(ctx, Value::known(Fr::from(*val as u64))))
            .collect_vec();
        println!("assigned_commitment: {:?}", assigned_commitment);
        let features_bits = self.bytes2bits(ctx, &assigned_features);
        println!("features_bits: {:?}", features_bits);
        let errors_bits = self.bytes2bits(ctx, &assigned_errors);
        println!("errors_bits: {:?}", errors_bits);
        let commitment_bits = self.bytes2bits(ctx, &assigned_commitment);

        // 1. word errored = features XOR commitment
        let w_e = features_bits
            .iter()
            .zip(commitment_bits.iter())
            .map(|(f, c)| self.xor(ctx, &f, &c))
            .collect_vec();
        println!("w_e: {:?}", w_e);
        // 2. word = word errored XOR error
        let word_bits = w_e
            .iter()
            .zip(errors_bits.iter())
            .map(|(y, e)| self.xor(ctx, &y, &e))
            .collect_vec();
        println!("word_bits: {:?}", word_bits);
        let word_bytes = word_bits
            .chunks(8)
            .map(|bits| {
                let mut byte = gate.load_zero(ctx);
                for (idx, bit) in bits.into_iter().enumerate() {
                    byte = gate.mul_add(
                        ctx,
                        QuantumCell::Existing(&bit),
                        QuantumCell::Constant(Fr::from(1u64 << idx)),
                        QuantumCell::Existing(&byte),
                    );
                }
                byte
            })
            .collect_vec();
        println!("word_bytes: {:?}", word_bytes);
        // 3. |e| < t
        let mut e_weight = gate.load_zero(ctx);
        for (idx, bit) in errors_bits.iter().enumerate() {
            e_weight = gate.add(
                ctx,
                QuantumCell::Existing(&e_weight),
                QuantumCell::Existing(&bit),
            );
        }
        println!("e_weight: {:?}", e_weight);
        range.check_less_than_safe(ctx, &e_weight, self.error_threshold);
        let word_values = features
            .iter()
            .zip(errors.iter())
            .zip(commitment.iter())
            .map(|((f, e), c)| f ^ e ^ c)
            .collect_vec();
        println!("word_values: {:?}", word_values);
        let assigned_feature_hash =
            poseidon.hash_elements(ctx, self.gate(), &word_bytes)?.0[0].clone();
        println!("assigned_feature_hash: {:?}", assigned_feature_hash);
        Ok(FuzzyCommitmentResult {
            assigned_commitment,
            assigned_feature_hash,
            assigned_word: word_bytes,
            word_value: word_values,
        })
    }

    pub fn range(&self) -> &RangeConfig<Fr> {
        &self.range_config
    }

    pub fn gate(&self) -> &FlexGateConfig<Fr> {
        self.range().gate()
    }

    pub fn new_context<'a, 'b>(&'b self, region: Region<'a, Fr>) -> Context<'a, Fr> {
        Context::new(
            region,
            ContextParams {
                max_rows: self.gate().max_rows,
                num_context_ids: 1,
                fixed_columns: self.gate().constants.clone(),
            },
        )
    }
    pub fn finalize(&self, ctx: &mut Context<Fr>) {
        self.range().finalize(ctx);
    }

    fn bytes2bits<'v: 'a, 'a>(
        &self,
        ctx: &mut Context<'v, Fr>,
        assigned_bytes: &[AssignedValue<'a, Fr>],
    ) -> Vec<AssignedValue<'a, Fr>> {
        let gate = self.gate();
        let bits = assigned_bytes
            .into_iter()
            .flat_map(|byte| gate.num_to_bits(ctx, byte, 8))
            .collect_vec();
        assert_eq!(assigned_bytes.len() * 8, bits.len());
        bits
    }

    fn xor<'v: 'a, 'a>(
        &self,
        ctx: &mut Context<'v, Fr>,
        a: &AssignedValue<'a, Fr>,
        b: &AssignedValue<'a, Fr>,
    ) -> AssignedValue<'a, Fr> {
        let gate = self.gate();
        let a_not = gate.not(ctx, QuantumCell::Existing(&a));
        let b_not = gate.not(ctx, QuantumCell::Existing(&b));
        let ab_not = gate.and(
            ctx,
            QuantumCell::Existing(&a),
            QuantumCell::Existing(&b_not),
        );
        let ba_not = gate.and(
            ctx,
            QuantumCell::Existing(&b),
            QuantumCell::Existing(&a_not),
        );
        gate.or(
            ctx,
            QuantumCell::Existing(&ab_not),
            QuantumCell::Existing(&ba_not),
        )
    }
}
