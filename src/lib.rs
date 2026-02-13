pub mod wallet;

use std::env;

pub trait HasOwWalletFields {
    fn use_kms(&self) -> bool;
    fn rpc_url(&self) -> String;
    fn private_key(&self) -> Option<String>;
    fn signer_kms_id(&self) -> Option<String>;
}

pub struct OwWalletConfig {
    pub use_kms: bool,
    pub rpc_url: String,
    pub private_key: Option<String>,
    pub signer_kms_id: Option<String>,
}

impl OwWalletConfig {
    pub fn build() -> anyhow::Result<Self> {
        let rpc_url = Self::get_env_var("RPC_URL");
        let mut signer_kms_id = None;
        let mut private_key = None;
        let use_kms = matches!(
            std::env::var("USE_KMS")
                .unwrap_or_else(|_| "false".to_string())
                .as_str(),
            "1" | "true"
        );

        if use_kms {
            signer_kms_id = Some(Self::get_env_var("SIGNER_KMS_ID"));
        } else {
            private_key = Some(Self::get_env_var("PRIVATE_KEY"));
        }

        Ok(Self {
            use_kms,
            rpc_url,
            private_key,
            signer_kms_id,
        })
    }

    fn get_env_var(key: &str) -> String {
        env::var(key).expect(format!("Missing env variable: {key}").as_str())
    }

    pub fn from<C: HasOwWalletFields>(config_source: &C) -> anyhow::Result<Self> {
        Ok(Self {
            use_kms: config_source.use_kms(),
            rpc_url: config_source.rpc_url(),
            private_key: config_source.private_key(),
            signer_kms_id: config_source.signer_kms_id(),
        })
    }
    fn try_private_key(&self) -> anyhow::Result<&String> {
        if self.use_kms == false {
            self.private_key
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Missing private_key"))
        } else {
            return Err(anyhow::anyhow!(
                "private_key not available with USE_KMS=true flag"
            ));
        }
    }

    fn try_signer_kms_id(&self) -> anyhow::Result<&String> {
        if self.use_kms == true {
            self.signer_kms_id
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Missing signer_kms_id"))
        } else {
            return Err(anyhow::anyhow!(
                "signer_kms_id not available without USE_KMS=true flag"
            ));
        }
    }
}
