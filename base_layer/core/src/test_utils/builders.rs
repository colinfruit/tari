// Copyright 2019. The Tari Project
//
// Redistribution and use in source and binary forms, with or without modification, are permitted provided that the
// following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following
// disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the
// following disclaimer in the documentation and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote
// products derived from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES,
// INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
// WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE
// USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
//

use crate::{
    blocks::{aggregated_body::AggregateBody, block::Block, blockheader::BlockHeader},
    tari_amount::MicroTari,
    transaction::{KernelBuilder, OutputFeatures, Transaction, TransactionInput, TransactionKernel, TransactionOutput},
    transaction_protocol::{build_challenge, TransactionMetadata},
    types::{PrivateKey, PublicKey, RangeProof, Signature, SignatureHash, COMMITMENT_FACTORY},
};
use merklemountainrange::merklemountainrange::MerkleMountainRange;
use tari_crypto::{
    commitment::HomomorphicCommitmentFactory,
    keys::{PublicKey as PK, SecretKey},
};

/// Create an unconfirmed transaction for testing with a valid fee, unique access_sig, random inputs and outputs, the
/// transaction is only partially constructed
pub fn create_test_tx(
    amount: MicroTari,
    fee: MicroTari,
    lock_height: u64,
    input_count: usize,
    output_count: usize,
) -> Transaction
{
    let mut rng = rand::OsRng::new().unwrap();
    let kernel = create_test_kernel(fee, lock_height);
    let mut body = AggregateBody::empty();
    body.kernels.push(kernel);

    for _ in 0..input_count {
        let input = TransactionInput::new(
            OutputFeatures::default(),
            COMMITMENT_FACTORY.commit(&PrivateKey::random(&mut rng), &amount.into()),
        );
        body.inputs.push(input);
    }

    for _ in 0..output_count {
        let output = TransactionOutput::new(
            OutputFeatures::default(),
            COMMITMENT_FACTORY.commit(&PrivateKey::random(&mut rng), &MicroTari(10).into()),
            RangeProof { 0: Vec::new() },
        );
        body.outputs.push(output);
    }

    Transaction {
        offset: PrivateKey::random(&mut rng),
        body,
    }
}

/// Create a transaction kernel with the given fee, using random keys to generate the signature
pub fn create_test_kernel(fee: MicroTari, lock_height: u64) -> TransactionKernel {
    let mut rng = rand::OsRng::new().unwrap();
    let tx_meta = TransactionMetadata { fee, lock_height: 0 };
    let key = PrivateKey::random(&mut rng);
    let r = PrivateKey::random(&mut rng);
    let e = build_challenge(&PublicKey::from_secret_key(&r), &tx_meta);
    let excess = COMMITMENT_FACTORY.commit_value(&key, 0);
    let s = Signature::sign(key.clone(), r, &e).unwrap();
    KernelBuilder::new()
        .with_fee(fee)
        .with_lock_height(lock_height)
        .with_excess(&excess)
        .with_signature(&s)
        .build()
        .unwrap()
}

/// Create a partially constructed block using the provided set of transactions
pub fn create_test_block(block_height: u64, transactions: Vec<Transaction>) -> Block {
    let mut header = BlockHeader::new(0);
    header.height = block_height;
    let mut body = AggregateBody::empty();
    transactions.iter().for_each(|tx| {
        body.kernels.push(tx.body.kernels[0].clone());
        body.inputs.append(&mut tx.body.inputs.clone());
        body.outputs.append(&mut tx.body.outputs.clone());
    });

    Block { header, body }
}

/// Create a partially constructed utxo set using the outputs of a test block
pub fn create_test_utxos(published_block: &Block) -> MerkleMountainRange<TransactionInput, SignatureHash> {
    let mut utxos = MerkleMountainRange::<TransactionInput, SignatureHash>::new();
    extend_test_utxos(&mut utxos, published_block);
    utxos
}

/// Extend an existing partially constructed utxo set using the outputs of the provided test block
pub fn extend_test_utxos(utxos: &mut MerkleMountainRange<TransactionInput, SignatureHash>, published_block: &Block) {
    for output in &published_block.body.outputs {
        let input = TransactionInput::from(output.clone());
        if !utxos.contains(&input) {
            utxos.push(input).unwrap();
        }
    }
}