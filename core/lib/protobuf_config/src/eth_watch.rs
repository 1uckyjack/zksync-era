use anyhow::Context as _;
use zksync_config::configs;
use zksync_protobuf::{repr::ProtoRepr, required};

use crate::proto::eth_watch as proto;

impl ProtoRepr for proto::EthWatch {
    type Type = configs::ETHWatchConfig;
    fn read(&self) -> anyhow::Result<Self::Type> {
        Ok(Self::Type {
            confirmations_for_eth_event: self.confirmations_for_eth_event,
            eth_node_poll_interval: *required(&self.eth_node_poll_interval)
                .context("eth_node_poll_interval")?,
        })
    }

    fn build(this: &Self::Type) -> Self {
        Self {
            confirmations_for_eth_event: this.confirmations_for_eth_event,
            eth_node_poll_interval: Some(this.eth_node_poll_interval),
        }
    }
}
