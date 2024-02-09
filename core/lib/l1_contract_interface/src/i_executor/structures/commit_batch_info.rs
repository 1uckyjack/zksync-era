use zkevm_test_harness_1_4_1::kzg::KzgSettings;
use zksync_types::{
    commitment::L1BatchWithMetadata,
    ethabi::Token,
    web3::{contract::Error as Web3ContractError, error::Error as Web3ApiError},
    U256,
};

use crate::{i_executor::commit::kzg::KzgInfo, Tokenizable};

use once_cell::sync::Lazy;

/// Loads KZG settings from the file system.
pub fn load_kzg_settings() -> KzgSettings {
    static KZG_SETTINGS: Lazy<KzgSettings> = Lazy::new(|| {
        let zksync_home = std::env::var("ZKSYNC_HOME").unwrap_or_else(|_| ".".into());
        let path = std::path::Path::new(&zksync_home).join("trusted_setup.json");
        KzgSettings::new(path.to_str().unwrap())
    });
    KZG_SETTINGS.clone()
}

/// Encoding for `CommitBatchInfo` from `IExecutor.sol`
#[derive(Debug)]
pub struct CommitBatchInfo<'a>(pub &'a L1BatchWithMetadata);

impl<'a> Tokenizable for CommitBatchInfo<'a> {
    fn from_token(_token: Token) -> Result<Self, Web3ContractError>
    where
        Self: Sized,
    {
        // Currently there is no need to decode this struct.
        // We still want to implement `Tokenizable` trait for it, so that *once* it's needed
        // the implementation is provided here and not in some other inconsistent way.
        Err(Web3ContractError::Api(Web3ApiError::Decoder(
            "Not implemented".to_string(),
        )))
    }

    fn into_token(self) -> Token {
        if self.0.header.protocol_version.unwrap().is_pre_boojum() {
            Token::Tuple(vec![
                Token::Uint(U256::from(self.0.header.number.0)),
                Token::Uint(U256::from(self.0.header.timestamp)),
                Token::Uint(U256::from(self.0.metadata.rollup_last_leaf_index)),
                Token::FixedBytes(self.0.metadata.merkle_root_hash.as_bytes().to_vec()),
                Token::Uint(U256::from(self.0.header.l1_tx_count)),
                Token::FixedBytes(self.0.metadata.l2_l1_merkle_root.as_bytes().to_vec()),
                Token::FixedBytes(
                    self.0
                        .header
                        .priority_ops_onchain_data_hash()
                        .as_bytes()
                        .to_vec(),
                ),
                Token::Bytes(self.0.metadata.initial_writes_compressed.clone()),
                Token::Bytes(self.0.metadata.repeated_writes_compressed.clone()),
                Token::Bytes(self.0.metadata.l2_l1_messages_compressed.clone()),
                Token::Array(
                    self.0
                        .header
                        .l2_to_l1_messages
                        .iter()
                        .map(|message| Token::Bytes(message.to_vec()))
                        .collect(),
                ),
                Token::Array(
                    self.0
                        .factory_deps
                        .iter()
                        .map(|bytecode| Token::Bytes(bytecode.to_vec()))
                        .collect(),
                ),
            ])
        } else {
            let pubdata = self
                .0
                .header
                .pubdata_input
                .clone()
                .unwrap_or(self.0.construct_pubdata());
            let kzg_info = KzgInfo::new(&load_kzg_settings(), pubdata);
            let mut pubdata_commitment = kzg_info.to_pubdata_commitment().to_vec();
            pubdata_commitment.insert(0, 1u8);

            Token::Tuple(vec![
                // `batchNumber`
                Token::Uint(U256::from(self.0.header.number.0)),
                // `timestamp`
                Token::Uint(U256::from(self.0.header.timestamp)),
                // `indexRepeatedStorageChanges`
                Token::Uint(U256::from(self.0.metadata.rollup_last_leaf_index)),
                // `newStateRoot`
                Token::FixedBytes(self.0.metadata.merkle_root_hash.as_bytes().to_vec()),
                // `numberOfLayer1Txs`
                Token::Uint(U256::from(self.0.header.l1_tx_count)),
                // `priorityOperationsHash`
                Token::FixedBytes(
                    self.0
                        .header
                        .priority_ops_onchain_data_hash()
                        .as_bytes()
                        .to_vec(),
                ),
                // `bootloaderHeapInitialContentsHash`
                Token::FixedBytes(
                    self.0
                        .metadata
                        .bootloader_initial_content_commitment
                        .unwrap()
                        .as_bytes()
                        .to_vec(),
                ),
                // `eventsQueueStateHash`
                Token::FixedBytes(
                    self.0
                        .metadata
                        .events_queue_commitment
                        .unwrap()
                        .as_bytes()
                        .to_vec(),
                ),
                // `systemLogs`
                Token::Bytes(self.0.metadata.l2_l1_messages_compressed.clone()),
                // `totalL2ToL1Pubdata`
                Token::Bytes(pubdata_commitment),
            ])
        }
    }
}