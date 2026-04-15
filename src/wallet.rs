use crate::OwWalletConfig;
use alloy::eips::BlockId;
use alloy::network::EthereumWallet;
use alloy::primitives::{Address, B256};
use alloy::providers::fillers::{
    BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller,
};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::signers::Signer;
use alloy::signers::local::PrivateKeySigner;
use alloy::sol_types::SolStruct;
use alloy_signer_aws::AwsSigner;
use alloy_sol_types::Eip712Domain;
use anyhow::Context;
use aws_config::BehaviorVersion;
use aws_config::meta::region::RegionProviderChain;

pub struct OwWallet {
    pub use_kms: bool,
    pub wallet: EthereumWallet,
    pub chain_id: u64,
    pub rpc_url: String,
    pub provider: FillProvider<
        JoinFill<
            alloy::providers::Identity,
            JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>,
        >,
        alloy::providers::RootProvider,
    >,
    aws_signer: Option<AwsSigner>,
    private_key_signer: Option<PrivateKeySigner>,
}

impl OwWallet {
    pub async fn build(config: &OwWalletConfig) -> anyhow::Result<Self> {
        let wallet: EthereumWallet;
        let mut aws_signer = None;
        let mut private_key_signer: Option<PrivateKeySigner> = None;

        let provider = ProviderBuilder::new()
            .connect(&config.rpc_url.as_str())
            .await?;
        let chain_id = provider.get_chain_id().await?;

        if config.use_kms {
            let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
            let aws_main_config = aws_config::defaults(BehaviorVersion::latest())
                .region(region_provider)
                .load()
                .await;

            let client = aws_sdk_kms::Client::new(&aws_main_config);

            let key_id = config.try_signer_kms_id()?;

            let signer = AwsSigner::new(client, key_id.to_string(), Some(chain_id))
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
            provider,
            chain_id,
            rpc_url: config.rpc_url.clone(),
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

    pub fn get_address(&self) -> anyhow::Result<Address> {
        if self.use_kms {
            Ok(self.try_aws_signer()?.address())
        } else {
            Ok(self.try_private_key_signer()?.address())
        }
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

    pub async fn sign_hash(&self, hash: B256) -> anyhow::Result<alloy::signers::Signature> {
        let signature;
        if self.use_kms {
            let aws_signer = self.try_aws_signer()?;
            signature = aws_signer.sign_hash(&hash).await?;
        } else {
            let private_key_signer = self.try_private_key_signer()?;
            signature = private_key_signer.sign_hash(&hash).await?;
        }
        Ok(signature)
    }

    pub async fn sign_typed_data<T: SolStruct + Send + Sync>(
        &self,
        payload: &T,
        domain: &Eip712Domain,
    ) -> anyhow::Result<alloy::signers::Signature> {
        let signature;
        if self.use_kms {
            let aws_signer = self.try_aws_signer()?;
            signature = aws_signer.sign_typed_data(payload, domain).await?;
        } else {
            let private_key_signer = self.try_private_key_signer()?;
            signature = private_key_signer.sign_typed_data(payload, domain).await?;
        }
        Ok(signature)
    }
}
