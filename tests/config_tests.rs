struct TestConfig {
    use_kms: bool,
    rpc_url: String,
    private_key: Option<String>,
    signer_kms_id: Option<String>,
}

impl ow_wallet::HasOwWalletFields for TestConfig {
    fn use_kms(&self) -> bool {
        self.use_kms
    }
    fn rpc_url(&self) -> String {
        self.rpc_url.clone()
    }
    fn private_key(&self) -> Option<String> {
        self.private_key.clone()
    }
    fn signer_kms_id(&self) -> Option<String> {
        self.signer_kms_id.clone()
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;
    use ow_wallet::OwWalletConfig;
    use serial_test::serial;
    use std::env;

    fn set_env_vars(vars: &Vec<(&str, &str)>) {
        for (key, value) in vars {
            unsafe {
                env::set_var(key, value);
            }
        }
    }
    fn clear_env_vars(vars: &Vec<(&str, &str)>) {
        for (key, _) in vars {
            unsafe {
                env::remove_var(key);
            }
        }
    }

    #[test]
    #[serial]
    fn test_config_build_with_private_key() {
        let env_vars = vec![
            ("USE_KMS", "false"),
            ("RPC_URL", "http://localhost:8545"),
            (
                "PRIVATE_KEY",
                "0x1234567890123456789012345678901234567890123456789012345678901234",
            ),
        ];
        set_env_vars(&env_vars);

        let config = OwWalletConfig::build().expect("Failed to build config");

        assert_eq!(config.use_kms, false);
        assert_eq!(config.rpc_url, "http://localhost:8545");
        assert!(config.private_key.is_some());
        assert!(config.signer_kms_id.is_none());
        clear_env_vars(&env_vars);
    }

    #[test]
    #[serial]
    fn test_config_build_with_kms() {
        let env_vars = vec![
            ("USE_KMS", "true"),
            ("RPC_URL", "http://localhost:8545"),
            ("SIGNER_KMS_ID", "12345678-1234-1234-1234-123456789012"),
        ];
        set_env_vars(&env_vars);
        let config = OwWalletConfig::build().expect("Failed to build config");

        assert_eq!(config.use_kms, true);
        assert_eq!(config.rpc_url, "http://localhost:8545");
        assert!(config.private_key.is_none());
        assert!(config.signer_kms_id.is_some());
    }

    #[test]
    #[serial]
    fn test_config_from_trait() {
        let test_config = TestConfig {
            use_kms: false,
            rpc_url: "http://test:8545".to_string(),
            private_key: Some("0x1234".to_string()),
            signer_kms_id: None,
        };

        let config = OwWalletConfig::from(&test_config).expect("Failed to create config");

        assert_eq!(config.use_kms, false);
        assert_eq!(config.rpc_url, "http://test:8545");
        assert_eq!(config.private_key, Some("0x1234".to_string()));
        assert!(config.signer_kms_id.is_none());
    }
}
