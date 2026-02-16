#[cfg(test)]
mod kms_wallet_tests {
    // Before running these tests:
    // 1. configure AWS access
    // 2. use .env.example file to create your .env file and provide your real KMS_KEY_ID and RPC_URL
    // 3. comment out `#[ignore]`

    use ow_wallet_adapter::{OwWalletConfig, wallet::OwWallet};
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    #[ignore]
    async fn test_wallet_build_with_kms() {
        dotenvy::dotenv().ok();

        let config = OwWalletConfig::build().unwrap();

        let wallet = OwWallet::build(&config).await;

        let wallet = wallet.unwrap();
        assert_eq!(wallet.use_kms, true);
        assert!(wallet.aws_signer.is_some());
        assert!(wallet.private_key_signer.is_none());
    }

    #[tokio::test]
    #[serial]
    #[ignore]
    async fn test_wallet_sign_message_with_kms() {
        dotenvy::dotenv().ok();

        let config = OwWalletConfig::build().unwrap();

        let wallet = OwWallet::build(&config).await.unwrap();

        let message = b"I like trains";
        let signature = wallet.sign_message(message).await.unwrap();
        let signature2 = wallet.sign_message(message).await.unwrap();

        let recovered_address = signature.recover_address_from_msg(message).unwrap();
        let recovered_address2 = signature2.recover_address_from_msg(message).unwrap();
        let wallet_address = wallet.wallet.default_signer().address();

        assert_eq!(
            recovered_address, recovered_address2,
            "Two recovered addresses doesn't match"
        );

        assert_eq!(
            wallet_address, recovered_address,
            "Wallet address doesn't match recovered address"
        );
    }
}
