use ferroflux_iam::TenantId;
use ferroflux_core::secrets::{DatabaseSecretStore, SecretStore};
use ferroflux_security::encryption::{decrypt, encrypt, get_or_create_master_key};
use ferroflux_core::store::database::PersistentStore;
use tokio::runtime::Runtime;

#[test]
fn test_secure_connection_flow() {
    // 1. Setup
    let _ = dotenv::dotenv();
    // Use in-memory DB
    let db_url = "sqlite::memory:";

    // Create a runtime for the test
    let rt = Runtime::new().unwrap();

    // 2. Initialize Store & Master Key (Async)
    let (store, master_key) = rt.block_on(async {
        let store = PersistentStore::new(db_url)
            .await
            .expect("Failed to init DB");
        // Force a known key or generated one
        let master_key = get_or_create_master_key().expect("Failed to get master key");
        (store, master_key)
    });

    // 3. Create Secret Store
    // No longer needs rt_handle!
    let secret_store = DatabaseSecretStore::new(store.clone(), master_key.clone());

    // 4. Simulate "Create Connection" (UI Action)
    let slug = "test-openai";
    let provider = "openai";
    let raw_json = r#"{"api_key": "sk-test-12345"}"#;

    // Encrypt
    let (ciphertext, nonce) = encrypt(raw_json.as_bytes(), &master_key).unwrap();

    // Save to DB (Async)
    let tenant_id = TenantId::from("default_tenant");
    rt.block_on(async {
        store
            .save_connection(
                &tenant_id,
                slug,
                slug,
                provider,
                &ciphertext,
                &nonce,
                "active",
            )
            .await
            .expect("Failed to save connection");
    });

    // 5. Verify "Detailed Retrieval" (Admin Action)
    // Manually fetch and decrypt to verify storage correctness
    let (stored_type, stored_cipher, stored_nonce, _, _) = rt.block_on(async {
        store
            .get_connection_by_slug(&tenant_id, slug)
            .await
            .unwrap()
            .expect("Connection not found in DB")
    });

    assert_eq!(stored_type, provider);
    assert_eq!(stored_cipher, ciphertext);
    assert_eq!(stored_nonce, nonce);

    let decrypted_bytes = decrypt(&stored_cipher, &master_key, &stored_nonce).unwrap();
    let decrypted_str = String::from_utf8(decrypted_bytes).unwrap();
    assert_eq!(decrypted_str, raw_json);

    // 6. Verify "Runtime Resolution" (Workflow Action)
    // resolve_connection is now async, so we must await it.
    let result_json = rt.block_on(async {
        secret_store
            .resolve_connection(&tenant_id, slug)
            .await
            .expect("Failed to resolve connection")
    });

    assert_eq!(result_json["api_key"], "sk-test-12345");

    println!("Start-to-End Secure Connection Test Passed!");
}
