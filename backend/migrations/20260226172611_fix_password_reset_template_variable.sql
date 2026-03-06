-- Fix password reset template variable name
-- Change 'username' to 'user_name' to match other templates

-- Update the template version with correct variable name
UPDATE email_template_versions 
SET 
    body_text = E'Hello {{user_name}},

We received a request to reset your password. Click the link below to create a new password:

{{reset_link}}

This link will expire in {{expiry_minutes}} minutes.

If you did not request this password reset, please ignore this email. Your password will remain unchanged.

Best regards,
{{site_name}}',
    body_html = E'<html><body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
<h2>Hello {{user_name}},</h2>
<p>We received a request to reset your password. Click the link below to create a new password:</p>
<p style="margin: 20px 0;">
<a href="{{reset_link}}" style="background: #00FFC2; color: #121213; padding: 12px 24px; text-decoration: none; border-radius: 4px; display: inline-block;">Reset Password</a>
</p>
<p style="color: #666; font-size: 14px;">This link will expire in {{expiry_minutes}} minutes.</p>
<p style="color: #666; font-size: 14px;">If you did not request this password reset, please ignore this email. Your password will remain unchanged.</p>
<br>
<p>Best regards,<br>{{site_name}}</p>
</body></html>',
    variables_schema_json = '[{"name":"user_name","required":true,"description":"User display name"},{"name":"email","required":true,"description":"User email"},{"name":"reset_link","required":true,"description":"Password reset URL"},{"name":"expiry_minutes","required":true,"description":"Expiry in minutes"},{"name":"site_name","required":true,"description":"Site name"}]'::jsonb
WHERE family_id = '00000000-0000-0000-0000-000000000106'::uuid;
