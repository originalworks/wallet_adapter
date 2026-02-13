use alloy::network::EthereumWallet;
use alloy::primitives::Address;
use alloy::providers::{Provider, ProviderBuilder};
use alloy::signers::Signer;
use alloy::signers::local::PrivateKeySigner;
use alloy_signer_aws::AwsSigner;
use anyhow::Context;
use aws_config::BehaviorVersion;
use aws_config::meta::region::RegionProviderChain;

use crate::OwWalletConfig;

pub struct OwWallet {
    pub use_kms: bool,
    pub aws_signer: Option<AwsSigner>,
    pub private_key_signer: Option<PrivateKeySigner>,
    pub wallet: EthereumWallet,
}

impl OwWallet {
    pub async fn build(config: &OwWalletConfig) -> anyhow::Result<Self> {
        let wallet: EthereumWallet;
        let mut aws_signer = None;
        let mut private_key_signer: Option<PrivateKeySigner> = None;
        if config.use_kms {
            let rpc_provider = ProviderBuilder::new()
                .connect(&config.rpc_url.as_str())
                .await?;
            let chain_id = rpc_provider.get_chain_id().await?;

            let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
            let aws_main_config = aws_config::defaults(BehaviorVersion::latest())
                .region(region_provider)
                .load()
                .await;

            let client = aws_sdk_kms::Client::new(&aws_main_config);

            let key_id = config.try_signer_kms_id()?;

            let chain_id = Some(chain_id);
            let signer = AwsSigner::new(client, key_id.to_string(), chain_id)
                .await
                .expect("Failed to initialize AwsSigner");

            let pubkey = signer.get_pubkey().await?;
            let address = Address::from_public_key(&pubkey);
            println!("Using KMS with address: {}", address);
            aws_signer = Some(signer.clone());
            wallet = EthereumWallet::from(signer);
        } else {
            let pk_signer: PrivateKeySigner = config
                .try_private_key()?
                .parse()
                .with_context(|| "Failed to parse PRIVATE_KEY")?;
            private_key_signer = Some(pk_signer.clone());
            wallet = EthereumWallet::from(pk_signer);
        }
        Ok(Self {
            use_kms: config.use_kms,
            aws_signer,
            private_key_signer,
            wallet,
        })
    }

    fn try_aws_signer(&self) -> anyhow::Result<&AwsSigner> {
        self.aws_signer
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing aws_signer"))
    }

    fn try_private_key_signer(&self) -> anyhow::Result<&PrivateKeySigner> {
        self.private_key_signer
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing private_key_signer"))
    }

    pub async fn sign_message(&self, message: &[u8]) -> anyhow::Result<alloy::signers::Signature> {
        let signature;
        if self.use_kms {
            let aws_signer = self.try_aws_signer()?;
            signature = aws_signer.sign_message(message).await?;
        } else {
            let private_key_signer = self.try_private_key_signer()?;
            signature = private_key_signer.sign_message(message).await?;
        }
        Ok(signature)
    }
}
