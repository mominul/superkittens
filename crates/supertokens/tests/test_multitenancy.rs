mod common;

use serial_test::serial;

use supertokens::querier::Querier;
use supertokens::recipe::multitenancy::interfaces::RecipeInterface;
use supertokens::recipe::multitenancy::recipe_implementation::RecipeImplementationImpl;
use supertokens::recipe::multitenancy::types::*;

fn make_mt_impl() -> RecipeImplementationImpl {
    let querier = Querier::get_instance(Some("multitenancy".to_string())).unwrap();
    RecipeImplementationImpl { querier }
}

fn unique_tenant_id() -> String {
    format!("test-tenant-{}", uuid::Uuid::new_v4().simple())
}

// ===========================================================================
// Create Tenant
// ===========================================================================

#[tokio::test]
#[serial]
#[ignore = "requires SuperTokens Core with multitenancy license"]
#[should_panic(expected = "Cannot use feature: multi_tenancy")]
async fn test_create_tenant() {
    common::reset();
    common::init_with_session().unwrap();

    let mt = make_mt_impl();
    let mut ctx = common::new_user_context();
    let tenant_id = unique_tenant_id();

    let result = mt
        .create_or_update_tenant(&tenant_id, None, &mut ctx)
        .await
        .unwrap();

    assert!(
        result.created_new,
        "First creation should set created_new to true"
    );

    // Cleanup
    mt.delete_tenant(&tenant_id, &mut ctx).await.unwrap();
    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires SuperTokens Core with multitenancy license"]
#[should_panic(expected = "Cannot use feature: multi_tenancy")]
async fn test_create_tenant_twice() {
    common::reset();
    common::init_with_session().unwrap();

    let mt = make_mt_impl();
    let mut ctx = common::new_user_context();
    let tenant_id = unique_tenant_id();

    let result1 = mt
        .create_or_update_tenant(&tenant_id, None, &mut ctx)
        .await
        .unwrap();
    assert!(
        result1.created_new,
        "First creation should set created_new to true"
    );

    let result2 = mt
        .create_or_update_tenant(&tenant_id, None, &mut ctx)
        .await
        .unwrap();
    assert!(
        !result2.created_new,
        "Second creation should set created_new to false"
    );

    // Cleanup
    mt.delete_tenant(&tenant_id, &mut ctx).await.unwrap();
    common::reset();
}

// ===========================================================================
// Get Tenant
// ===========================================================================

#[tokio::test]
#[serial]
#[ignore = "requires SuperTokens Core with multitenancy license"]
#[should_panic(expected = "Cannot use feature: multi_tenancy")]
async fn test_get_tenant() {
    common::reset();
    common::init_with_session().unwrap();

    let mt = make_mt_impl();
    let mut ctx = common::new_user_context();
    let tenant_id = unique_tenant_id();

    mt.create_or_update_tenant(&tenant_id, None, &mut ctx)
        .await
        .unwrap();

    let tenant = mt.get_tenant(&tenant_id, &mut ctx).await.unwrap();

    assert!(tenant.is_some(), "Tenant should exist after creation");
    let tenant = tenant.unwrap();
    assert_eq!(tenant.tenant_id, tenant_id);

    // Cleanup
    mt.delete_tenant(&tenant_id, &mut ctx).await.unwrap();
    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires SuperTokens Core with multitenancy license"]
async fn test_get_nonexistent_tenant() {
    common::reset();
    common::init_with_session().unwrap();

    let mt = make_mt_impl();
    let mut ctx = common::new_user_context();
    let tenant_id = unique_tenant_id();

    let tenant = mt.get_tenant(&tenant_id, &mut ctx).await.unwrap();

    assert!(tenant.is_none(), "Non-existent tenant should return None");

    common::reset();
}

// ===========================================================================
// List All Tenants
// ===========================================================================

#[tokio::test]
#[serial]
#[ignore = "requires SuperTokens Core with multitenancy license"]
async fn test_list_all_tenants() {
    common::reset();
    common::init_with_session().unwrap();

    let mt = make_mt_impl();
    let mut ctx = common::new_user_context();

    let result = mt.list_all_tenants(&mut ctx).await.unwrap();

    assert!(
        result.tenants.iter().any(|t| t.tenant_id == "public"),
        "The 'public' tenant should always be present"
    );

    common::reset();
}

// ===========================================================================
// Delete Tenant
// ===========================================================================

#[tokio::test]
#[serial]
#[ignore = "requires SuperTokens Core with multitenancy license"]
#[should_panic(expected = "Cannot use feature: multi_tenancy")]
async fn test_delete_tenant() {
    common::reset();
    common::init_with_session().unwrap();

    let mt = make_mt_impl();
    let mut ctx = common::new_user_context();
    let tenant_id = unique_tenant_id();

    mt.create_or_update_tenant(&tenant_id, None, &mut ctx)
        .await
        .unwrap();

    let result = mt.delete_tenant(&tenant_id, &mut ctx).await.unwrap();

    assert!(
        result.did_exist,
        "Deleting an existing tenant should set did_exist to true"
    );

    // Verify it's gone
    let tenant = mt.get_tenant(&tenant_id, &mut ctx).await.unwrap();
    assert!(tenant.is_none(), "Tenant should not exist after deletion");

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires SuperTokens Core with multitenancy license"]
async fn test_delete_nonexistent_tenant() {
    common::reset();
    common::init_with_session().unwrap();

    let mt = make_mt_impl();
    let mut ctx = common::new_user_context();
    let tenant_id = unique_tenant_id();

    let result = mt.delete_tenant(&tenant_id, &mut ctx).await.unwrap();

    assert!(
        !result.did_exist,
        "Deleting a non-existent tenant should set did_exist to false"
    );

    common::reset();
}

// ===========================================================================
// Create Tenant with Config
// ===========================================================================

#[tokio::test]
#[serial]
#[ignore = "requires SuperTokens Core with multitenancy license"]
#[should_panic(expected = "Cannot use feature: multi_tenancy")]
async fn test_create_tenant_with_config() {
    common::reset();
    common::init_with_session().unwrap();

    let mt = make_mt_impl();
    let mut ctx = common::new_user_context();
    let tenant_id = unique_tenant_id();

    let config = TenantConfigCreateOrUpdate {
        first_factors: Some(Some(vec!["emailpassword".to_string()])),
        ..Default::default()
    };

    let result = mt
        .create_or_update_tenant(&tenant_id, Some(&config), &mut ctx)
        .await
        .unwrap();

    assert!(result.created_new, "Should create a new tenant");

    let tenant = mt
        .get_tenant(&tenant_id, &mut ctx)
        .await
        .unwrap()
        .expect("Tenant should exist after creation");

    assert_eq!(tenant.tenant_id, tenant_id);
    let first_factors = tenant.first_factors.expect("first_factors should be set");
    assert_eq!(first_factors, vec!["emailpassword".to_string()]);

    // Cleanup
    mt.delete_tenant(&tenant_id, &mut ctx).await.unwrap();
    common::reset();
}
