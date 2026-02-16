#[cfg(test)]
mod pk_wallet_tests {
    use ow_wallet_adapter::{OwWalletConfig, wallet::OwWallet};
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn test_wallet_build_with_private_key() {
        let config = OwWalletConfig {
            use_kms: false,
            rpc_url: "https://eth-mainnet.g.alchemy.com/v2/your-api-key".to_string(),
            private_key: Some(
                "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string(),
            ), // test private key from Hardhat
            signer_kms_id: None,
        };

        let wallet = OwWallet::build(&config).await;
        assert!(wallet.is_ok());

        let wallet = wallet.unwrap();
        assert_eq!(wallet.use_kms, false);
        assert!(wallet.aws_signer.is_none());
        assert!(wallet.private_key_signer.is_some());
    }

    #[tokio::test]
    async fn test_wallet_sign_message_with_private_key() {
        let config = OwWalletConfig {
            use_kms: false,
            rpc_url: "https://eth-mainnet.g.alchemy.com/v2/your-api-key".to_string(),
            private_key: Some(
                "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string(),
            ),
            signer_kms_id: None,
        };

        let wallet = OwWallet::build(&config)
            .await
            .expect("Failed to build wallet");

        let message = b"I like trains";
        let signature = wallet.sign_message(message).await.unwrap();
        let signature2 = wallet.sign_message(message).await.unwrap();

        assert_eq!(signature.to_string(), signature2.to_string());

        let recovered_address = signature.recover_address_from_msg(message).unwrap();
        let wallet_address = wallet.wallet.default_signer().address();

        assert_eq!(
            wallet_address, recovered_address,
            "Wallet address doesn't match recovered address"
        );
    }
}
