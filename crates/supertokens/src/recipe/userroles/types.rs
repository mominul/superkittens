/// Result types for user roles operations.

#[derive(Debug, Clone)]
pub enum AddRoleToUserResult {
    Ok { did_user_already_have_role: bool },
    UnknownRole,
}

#[derive(Debug, Clone)]
pub enum RemoveUserRoleResult {
    Ok { did_user_have_role: bool },
    UnknownRole,
}

#[derive(Debug, Clone)]
pub struct GetRolesForUserOkResult {
    pub roles: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum GetUsersThatHaveRoleResult {
    Ok { users: Vec<String> },
    UnknownRole,
}

#[derive(Debug, Clone)]
pub struct CreateNewRoleOrAddPermissionsOkResult {
    pub created_new_role: bool,
}

#[derive(Debug, Clone)]
pub enum GetPermissionsForRoleResult {
    Ok { permissions: Vec<String> },
    UnknownRole,
}

#[derive(Debug, Clone)]
pub enum RemovePermissionsFromRoleResult {
    Ok,
    UnknownRole,
}

#[derive(Debug, Clone)]
pub struct GetRolesThatHavePermissionOkResult {
    pub roles: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DeleteRoleOkResult {
    pub did_role_exist: bool,
}

#[derive(Debug, Clone)]
pub struct GetAllRolesOkResult {
    pub roles: Vec<String>,
}
