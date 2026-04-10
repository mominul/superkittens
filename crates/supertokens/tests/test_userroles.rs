mod common;

use serial_test::serial;

use supertokens::querier::Querier;
use supertokens::recipe::userroles::interfaces::RecipeInterface;
use supertokens::recipe::userroles::recipe_implementation::RecipeImplementationImpl;
use supertokens::recipe::userroles::types::*;

fn make_userroles_impl() -> RecipeImplementationImpl {
    let querier = Querier::get_instance(Some("userroles".to_string())).unwrap();
    RecipeImplementationImpl { querier }
}

fn unique_role() -> String {
    format!("role-{}", uuid::Uuid::new_v4().simple())
}

// ===========================================================================
// Create New Role or Add Permissions
// (ported from test_create_new_role_or_add_permissions.py)
// ===========================================================================

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_create_new_role() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_userroles_impl();
    let mut ctx = common::new_user_context();
    let role = unique_role();

    let result = recipe
        .create_new_role_or_add_permissions(&role, &[], &mut ctx)
        .await
        .unwrap();

    assert!(result.created_new_role, "Should create a new role");

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_create_new_role_twice() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_userroles_impl();
    let mut ctx = common::new_user_context();
    let role = unique_role();

    let result1 = recipe
        .create_new_role_or_add_permissions(&role, &[], &mut ctx)
        .await
        .unwrap();
    assert!(result1.created_new_role);

    let result2 = recipe
        .create_new_role_or_add_permissions(&role, &[], &mut ctx)
        .await
        .unwrap();
    assert!(
        !result2.created_new_role,
        "Second call should not create a new role"
    );

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_create_new_role_with_permissions() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_userroles_impl();
    let mut ctx = common::new_user_context();
    let role = unique_role();

    let perms = vec!["read".to_string(), "write".to_string()];
    let result = recipe
        .create_new_role_or_add_permissions(&role, &perms, &mut ctx)
        .await
        .unwrap();
    assert!(result.created_new_role);

    // Verify permissions were added
    let perms_result = recipe
        .get_permissions_for_role(&role, &mut ctx)
        .await
        .unwrap();
    match perms_result {
        GetPermissionsForRoleResult::Ok { permissions } => {
            assert!(permissions.contains(&"read".to_string()));
            assert!(permissions.contains(&"write".to_string()));
        }
        _ => panic!("Expected Ok"),
    }

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_add_permissions_to_existing_role() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_userroles_impl();
    let mut ctx = common::new_user_context();
    let role = unique_role();

    // Create role with initial perms
    recipe
        .create_new_role_or_add_permissions(&role, &["read".to_string()], &mut ctx)
        .await
        .unwrap();

    // Add more perms
    let result = recipe
        .create_new_role_or_add_permissions(&role, &["write".to_string()], &mut ctx)
        .await
        .unwrap();
    assert!(!result.created_new_role);

    // Verify both permissions exist
    let perms_result = recipe
        .get_permissions_for_role(&role, &mut ctx)
        .await
        .unwrap();
    match perms_result {
        GetPermissionsForRoleResult::Ok { permissions } => {
            assert!(permissions.contains(&"read".to_string()));
            assert!(permissions.contains(&"write".to_string()));
        }
        _ => panic!("Expected Ok"),
    }

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_add_duplicate_permission() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_userroles_impl();
    let mut ctx = common::new_user_context();
    let role = unique_role();

    recipe
        .create_new_role_or_add_permissions(&role, &["read".to_string()], &mut ctx)
        .await
        .unwrap();

    // Adding same permission again should not error
    let result = recipe
        .create_new_role_or_add_permissions(&role, &["read".to_string()], &mut ctx)
        .await
        .unwrap();
    assert!(!result.created_new_role);

    // Should still have only one "read"
    let perms_result = recipe
        .get_permissions_for_role(&role, &mut ctx)
        .await
        .unwrap();
    match perms_result {
        GetPermissionsForRoleResult::Ok { permissions } => {
            assert_eq!(
                permissions.iter().filter(|p| *p == "read").count(),
                1,
                "Should not duplicate permissions"
            );
        }
        _ => panic!("Expected Ok"),
    }

    common::reset();
}

// ===========================================================================
// Add Role to User
// (ported from test_add_role_to_user.py)
// ===========================================================================

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_add_new_role_to_user() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_userroles_impl();
    let mut ctx = common::new_user_context();
    let role = unique_role();

    // Create role first
    recipe
        .create_new_role_or_add_permissions(&role, &[], &mut ctx)
        .await
        .unwrap();

    let user_id = uuid::Uuid::new_v4().to_string();
    let result = recipe
        .add_role_to_user(&user_id, &role, "public", &mut ctx)
        .await
        .unwrap();

    match result {
        AddRoleToUserResult::Ok {
            did_user_already_have_role,
        } => {
            assert!(
                !did_user_already_have_role,
                "User should not already have role"
            );
        }
        _ => panic!("Expected Ok"),
    }

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_add_duplicate_role_to_user() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_userroles_impl();
    let mut ctx = common::new_user_context();
    let role = unique_role();

    recipe
        .create_new_role_or_add_permissions(&role, &[], &mut ctx)
        .await
        .unwrap();

    let user_id = uuid::Uuid::new_v4().to_string();
    recipe
        .add_role_to_user(&user_id, &role, "public", &mut ctx)
        .await
        .unwrap();

    // Add same role again
    let result = recipe
        .add_role_to_user(&user_id, &role, "public", &mut ctx)
        .await
        .unwrap();

    match result {
        AddRoleToUserResult::Ok {
            did_user_already_have_role,
        } => {
            assert!(did_user_already_have_role, "User should already have role");
        }
        _ => panic!("Expected Ok"),
    }

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_add_unknown_role_to_user() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_userroles_impl();
    let mut ctx = common::new_user_context();

    let user_id = uuid::Uuid::new_v4().to_string();
    let result = recipe
        .add_role_to_user(&user_id, &unique_role(), "public", &mut ctx)
        .await
        .unwrap();

    assert!(
        matches!(result, AddRoleToUserResult::UnknownRole),
        "Should return UnknownRole"
    );

    common::reset();
}

// ===========================================================================
// Get Roles for User
// (ported from test_get_roles_for_user.py)
// ===========================================================================

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_get_roles_for_user() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_userroles_impl();
    let mut ctx = common::new_user_context();
    let role1 = unique_role();
    let role2 = unique_role();

    // Create roles
    recipe
        .create_new_role_or_add_permissions(&role1, &[], &mut ctx)
        .await
        .unwrap();
    recipe
        .create_new_role_or_add_permissions(&role2, &[], &mut ctx)
        .await
        .unwrap();

    let user_id = uuid::Uuid::new_v4().to_string();
    recipe
        .add_role_to_user(&user_id, &role1, "public", &mut ctx)
        .await
        .unwrap();
    recipe
        .add_role_to_user(&user_id, &role2, "public", &mut ctx)
        .await
        .unwrap();

    let result = recipe
        .get_roles_for_user(&user_id, "public", &mut ctx)
        .await
        .unwrap();

    assert_eq!(result.roles.len(), 2);
    assert!(result.roles.contains(&role1));
    assert!(result.roles.contains(&role2));

    common::reset();
}

// ===========================================================================
// Get Users That Have Role
// (ported from test_get_users_that_have_role.py)
// ===========================================================================

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_get_users_that_have_role() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_userroles_impl();
    let mut ctx = common::new_user_context();
    let role = unique_role();

    recipe
        .create_new_role_or_add_permissions(&role, &[], &mut ctx)
        .await
        .unwrap();

    let user1 = uuid::Uuid::new_v4().to_string();
    let user2 = uuid::Uuid::new_v4().to_string();
    recipe
        .add_role_to_user(&user1, &role, "public", &mut ctx)
        .await
        .unwrap();
    recipe
        .add_role_to_user(&user2, &role, "public", &mut ctx)
        .await
        .unwrap();

    let result = recipe
        .get_users_that_have_role(&role, "public", &mut ctx)
        .await
        .unwrap();

    match result {
        GetUsersThatHaveRoleResult::Ok { users } => {
            assert_eq!(users.len(), 2);
            assert!(users.contains(&user1));
            assert!(users.contains(&user2));
        }
        _ => panic!("Expected Ok"),
    }

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_get_users_for_unknown_role() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_userroles_impl();
    let mut ctx = common::new_user_context();

    let result = recipe
        .get_users_that_have_role(&unique_role(), "public", &mut ctx)
        .await
        .unwrap();

    assert!(
        matches!(result, GetUsersThatHaveRoleResult::UnknownRole),
        "Should return UnknownRole"
    );

    common::reset();
}

// ===========================================================================
// Remove User Role
// (ported from test_remove_user_role.py)
// ===========================================================================

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_remove_role_from_user() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_userroles_impl();
    let mut ctx = common::new_user_context();
    let role = unique_role();

    recipe
        .create_new_role_or_add_permissions(&role, &[], &mut ctx)
        .await
        .unwrap();

    let user_id = uuid::Uuid::new_v4().to_string();
    recipe
        .add_role_to_user(&user_id, &role, "public", &mut ctx)
        .await
        .unwrap();

    let result = recipe
        .remove_user_role(&user_id, &role, "public", &mut ctx)
        .await
        .unwrap();

    match result {
        RemoveUserRoleResult::Ok { did_user_have_role } => {
            assert!(did_user_have_role, "User should have had the role");
        }
        _ => panic!("Expected Ok"),
    }

    // Verify role was removed
    let roles = recipe
        .get_roles_for_user(&user_id, "public", &mut ctx)
        .await
        .unwrap();
    assert!(
        roles.roles.is_empty(),
        "User should have no roles after removal"
    );

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_remove_unassigned_role_from_user() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_userroles_impl();
    let mut ctx = common::new_user_context();
    let role = unique_role();

    recipe
        .create_new_role_or_add_permissions(&role, &[], &mut ctx)
        .await
        .unwrap();

    let user_id = uuid::Uuid::new_v4().to_string();
    let result = recipe
        .remove_user_role(&user_id, &role, "public", &mut ctx)
        .await
        .unwrap();

    match result {
        RemoveUserRoleResult::Ok { did_user_have_role } => {
            assert!(!did_user_have_role, "User should not have had the role");
        }
        _ => panic!("Expected Ok"),
    }

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_remove_nonexistent_role_from_user() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_userroles_impl();
    let mut ctx = common::new_user_context();

    let user_id = uuid::Uuid::new_v4().to_string();
    let result = recipe
        .remove_user_role(&user_id, &unique_role(), "public", &mut ctx)
        .await
        .unwrap();

    assert!(
        matches!(result, RemoveUserRoleResult::UnknownRole),
        "Should return UnknownRole"
    );

    common::reset();
}

// ===========================================================================
// Delete Role
// (ported from test_delete_role.py)
// ===========================================================================

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_create_and_delete_role() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_userroles_impl();
    let mut ctx = common::new_user_context();
    let role = unique_role();

    recipe
        .create_new_role_or_add_permissions(&role, &[], &mut ctx)
        .await
        .unwrap();

    // Assign to a user
    let user_id = uuid::Uuid::new_v4().to_string();
    recipe
        .add_role_to_user(&user_id, &role, "public", &mut ctx)
        .await
        .unwrap();

    // Delete the role
    let result = recipe.delete_role(&role, &mut ctx).await.unwrap();
    assert!(result.did_role_exist, "Role should have existed");

    // Verify role is gone from user
    let roles = recipe
        .get_roles_for_user(&user_id, "public", &mut ctx)
        .await
        .unwrap();
    assert!(
        roles.roles.is_empty(),
        "User should have no roles after deletion"
    );

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_delete_nonexistent_role() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_userroles_impl();
    let mut ctx = common::new_user_context();

    let result = recipe.delete_role(&unique_role(), &mut ctx).await.unwrap();
    assert!(!result.did_role_exist, "Role should not have existed");

    common::reset();
}

// ===========================================================================
// Get Permissions for Role
// (ported from test_get_permissions_for_role.py)
// ===========================================================================

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_get_permissions_for_role() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_userroles_impl();
    let mut ctx = common::new_user_context();
    let role = unique_role();

    let perms = vec![
        "read".to_string(),
        "write".to_string(),
        "delete".to_string(),
    ];
    recipe
        .create_new_role_or_add_permissions(&role, &perms, &mut ctx)
        .await
        .unwrap();

    let result = recipe
        .get_permissions_for_role(&role, &mut ctx)
        .await
        .unwrap();

    match result {
        GetPermissionsForRoleResult::Ok { permissions } => {
            assert_eq!(permissions.len(), 3);
            assert!(permissions.contains(&"read".to_string()));
            assert!(permissions.contains(&"write".to_string()));
            assert!(permissions.contains(&"delete".to_string()));
        }
        _ => panic!("Expected Ok"),
    }

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_get_permissions_for_nonexistent_role() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_userroles_impl();
    let mut ctx = common::new_user_context();

    let result = recipe
        .get_permissions_for_role(&unique_role(), &mut ctx)
        .await
        .unwrap();

    assert!(
        matches!(result, GetPermissionsForRoleResult::UnknownRole),
        "Should return UnknownRole"
    );

    common::reset();
}

// ===========================================================================
// Remove Permissions from Role
// (ported from test_remove_permissions_from_role.py)
// ===========================================================================

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_remove_permissions_from_role() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_userroles_impl();
    let mut ctx = common::new_user_context();
    let role = unique_role();

    let perms = vec![
        "read".to_string(),
        "write".to_string(),
        "delete".to_string(),
    ];
    recipe
        .create_new_role_or_add_permissions(&role, &perms, &mut ctx)
        .await
        .unwrap();

    // Remove "write"
    let result = recipe
        .remove_permissions_from_role(&role, &["write".to_string()], &mut ctx)
        .await
        .unwrap();

    assert!(matches!(result, RemovePermissionsFromRoleResult::Ok));

    // Verify
    let perms_result = recipe
        .get_permissions_for_role(&role, &mut ctx)
        .await
        .unwrap();
    match perms_result {
        GetPermissionsForRoleResult::Ok { permissions } => {
            assert_eq!(permissions.len(), 2);
            assert!(permissions.contains(&"read".to_string()));
            assert!(permissions.contains(&"delete".to_string()));
            assert!(!permissions.contains(&"write".to_string()));
        }
        _ => panic!("Expected Ok"),
    }

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_remove_permissions_from_unknown_role() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_userroles_impl();
    let mut ctx = common::new_user_context();

    let result = recipe
        .remove_permissions_from_role(&unique_role(), &["read".to_string()], &mut ctx)
        .await
        .unwrap();

    assert!(
        matches!(result, RemovePermissionsFromRoleResult::UnknownRole),
        "Should return UnknownRole"
    );

    common::reset();
}

// ===========================================================================
// Get Roles That Have Permission
// (ported from test_get_roles_that_have_permissions.py)
// ===========================================================================

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_get_roles_that_have_permission() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_userroles_impl();
    let mut ctx = common::new_user_context();
    let role1 = unique_role();
    let role2 = unique_role();
    // Use unique permission names to avoid pollution from other tests
    let perm_read = format!("read-{}", uuid::Uuid::new_v4().simple());
    let perm_write = format!("write-{}", uuid::Uuid::new_v4().simple());

    recipe
        .create_new_role_or_add_permissions(
            &role1,
            &[perm_read.clone(), perm_write.clone()],
            &mut ctx,
        )
        .await
        .unwrap();
    recipe
        .create_new_role_or_add_permissions(&role2, &[perm_read.clone()], &mut ctx)
        .await
        .unwrap();

    let result = recipe
        .get_roles_that_have_permission(&perm_read, &mut ctx)
        .await
        .unwrap();

    assert_eq!(result.roles.len(), 2);
    assert!(result.roles.contains(&role1));
    assert!(result.roles.contains(&role2));

    // perm_write should only match role1
    let result2 = recipe
        .get_roles_that_have_permission(&perm_write, &mut ctx)
        .await
        .unwrap();

    assert_eq!(result2.roles.len(), 1);
    assert!(result2.roles.contains(&role1));

    common::reset();
}

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_get_roles_for_unknown_permission() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_userroles_impl();
    let mut ctx = common::new_user_context();

    let result = recipe
        .get_roles_that_have_permission(&format!("nonexistent-{}", uuid::Uuid::new_v4()), &mut ctx)
        .await
        .unwrap();

    assert!(
        result.roles.is_empty(),
        "Should return empty list for unknown permission"
    );

    common::reset();
}

// ===========================================================================
// Get All Roles
// ===========================================================================

#[tokio::test]
#[serial]
#[ignore = "requires running SuperTokens Core"]
async fn test_get_all_roles() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_userroles_impl();
    let mut ctx = common::new_user_context();
    let role1 = unique_role();
    let role2 = unique_role();
    let role3 = unique_role();

    // Create some roles
    recipe
        .create_new_role_or_add_permissions(&role1, &[], &mut ctx)
        .await
        .unwrap();
    recipe
        .create_new_role_or_add_permissions(&role2, &[], &mut ctx)
        .await
        .unwrap();
    recipe
        .create_new_role_or_add_permissions(&role3, &[], &mut ctx)
        .await
        .unwrap();

    let result = recipe.get_all_roles(&mut ctx).await.unwrap();

    assert!(result.roles.contains(&role1));
    assert!(result.roles.contains(&role2));
    assert!(result.roles.contains(&role3));

    common::reset();
}

// ===========================================================================
// Multitenancy
// (ported from test_multitenancy.py)
// ===========================================================================

#[tokio::test]
#[serial]
#[ignore = "requires SuperTokens Core with multitenancy license"]
#[should_panic(expected = "Not found")]
async fn test_multitenancy_in_user_roles() {
    common::reset();
    common::init_with_session().unwrap();

    let recipe = make_userroles_impl();
    let mut ctx = common::new_user_context();
    let role = unique_role();

    // Create a role
    recipe
        .create_new_role_or_add_permissions(&role, &["read".to_string()], &mut ctx)
        .await
        .unwrap();

    let user_id = uuid::Uuid::new_v4().to_string();

    // Assign role in tenant "public"
    recipe
        .add_role_to_user(&user_id, &role, "public", &mut ctx)
        .await
        .unwrap();

    // User should have role in "public"
    let roles_public = recipe
        .get_roles_for_user(&user_id, "public", &mut ctx)
        .await
        .unwrap();
    assert!(roles_public.roles.contains(&role));

    // User should NOT have role in a different tenant
    let roles_other = recipe
        .get_roles_for_user(&user_id, "other_tenant", &mut ctx)
        .await
        .unwrap();
    assert!(
        roles_other.roles.is_empty(),
        "User should not have roles in other tenant"
    );

    common::reset();
}
