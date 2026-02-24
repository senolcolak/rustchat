-- Allow passwordless registration (password will be set via email link)
-- This modifies the trigger to allow pending_password_setup status

-- Drop the existing trigger
DROP TRIGGER IF EXISTS trg_check_local_user_password ON users;

-- Update the function to allow pending password setup
CREATE OR REPLACE FUNCTION check_local_user_password()
RETURNS TRIGGER AS $$
BEGIN
    -- Allow users without auth_provider to have NULL password if email is not yet verified
    -- This supports the passwordless registration flow where users set password via email link
    -- After email verification, they must have a password to login
    IF NEW.auth_provider IS NULL 
       AND (NEW.password_hash IS NULL OR NEW.password_hash = '') 
       AND NEW.email_verified = true THEN
        RAISE EXCEPTION 'Local users must have a password after email verification';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Recreate the trigger
CREATE TRIGGER trg_check_local_user_password
    BEFORE INSERT OR UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION check_local_user_password();

-- Add comment explaining the updated constraint
COMMENT ON FUNCTION check_local_user_password() IS 
    'Ensures local users (non-OAuth) have a password_hash after email verification, 
     while allowing OAuth users and pending-verification users to have NULL passwords';
