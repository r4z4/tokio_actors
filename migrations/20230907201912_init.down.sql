-- Add down migration script here
DROP TABLE IF EXISTS article_categories;
DROP TABLE IF EXISTS engagements;
DROP TABLE IF EXISTS reset_password_requests;
DROP TABLE IF EXISTS messages;
DROP TABLE IF EXISTS attachments;
DROP TABLE IF EXISTS offers;
DROP TABLE IF EXISTS loans;
DROP TABLE IF EXISTS credit_file;
DROP TABLE IF EXISTS servicers;
DROP TABLE IF EXISTS applications CASCADE;
DROP TABLE IF EXISTS borrowers;
DROP TABLE IF EXISTS locations;
DROP TABLE IF EXISTS contacts;
DROP TABLE IF EXISTS territories;
DROP TABLE IF EXISTS specialties;
DROP TABLE IF EXISTS mime_types;
DROP TABLE IF EXISTS entities; 
DROP TABLE IF EXISTS loan_purpose;
DROP TABLE IF EXISTS contact_submissions;
DROP TABLE IF EXISTS article_submissions;
DROP TABLE IF EXISTS states; 
DROP TABLE IF EXISTS user_sessions;
DROP TABLE IF EXISTS user_types;
DROP TABLE IF EXISTS user_settings;
DROP TABLE IF EXISTS user_details;
DROP TABLE IF EXISTS users;
-- This needs to be last
DROP TABLE IF EXISTS accounts;

DROP TRIGGER IF EXISTS user_settings_insert_trigger ON users;


DROP TYPE IF EXISTS consultant_specialty;
DROP TYPE IF EXISTS consultant_territory;
DROP TYPE IF EXISTS user_type;
DROP TYPE IF EXISTS state_abbr;
DROP TYPE IF EXISTS state_name;
DROP TYPE IF EXISTS us_territories;
DROP TYPE IF EXISTS attachment_channel;
DROP TYPE IF EXISTS mime_type;