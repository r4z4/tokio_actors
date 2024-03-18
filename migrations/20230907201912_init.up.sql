-- Add up migration script here

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- CREATE EXTENSION IF NOT EXISTS vector;
--DROP TABLE IF EXISTS accounts;
DROP TYPE IF EXISTS user_type;
DROP TYPE IF EXISTS specialty;
DROP TYPE IF EXISTS territory;
DROP TYPE IF EXISTS state_abbr;
DROP TYPE IF EXISTS state_name;
DROP TYPE IF EXISTS us_territories;
DROP TYPE IF EXISTS attachment_channel;
DROP TYPE IF EXISTS mime_type;

-- When I forget to add it to DOWN file
-- DROP TABLE IF EXISTS user_types;

-- CREATE TYPE consultant_specialty AS ENUM ('Insurance', 'Finance', 'Government');

-- CREATE TYPE mime_type AS ENUM ('application/pdf', 'application/json', 'video/mp4', 'image/jpeg', 'image/svg+xml', 'image/gif', 'image/png');

-- CREATE TYPE attachment_channel AS ENUM ('Email', 'Upload');

-- CREATE TYPE consultant_territory AS ENUM ('Midwest', 'East', 'West', 'North', 'South');

-- CREATE TYPE state_abbr AS ENUM ('AL','AK','AZ','AR','CA','CO','CT','DE','FL','GA','HI','ID','IL','IN','IA','KS','KY','LA','ME','MD','MA',
--         'MI','MN','MS','MO','MT','NE','NV','NH','NJ','NM','NY','NC','ND','OH','OK','OR','PA','RI','SC','SD','TN',
--         'TX','UT','VT','VA','WA','WV','WI','WY','AS','GU','MP','PR','VI','DC');

-- CREATE TYPE state_name AS ENUM ('Alabama','Alaska','Arizona','Arkansas','California','Colorado','Connecticut','Delaware','Florida','Georgia',
        -- 'Hawaii','Idaho','Illinois','Indiana','Iowa','Kansas','Kentucky','Louisiana','Maine','Maryland','Massachusetts',
        -- 'Michigan','Minnesota','Mississippi','Missouri','Montana','Nebraska','Nevada','New_Hampshire','New_Jersey','New_Mexico',
        -- 'New_York','North_Carolina','North_Dakota','Ohio','Oklahoma','Oregon','Pennsylvania','Rhode_Island','South_Carolina',
        -- 'South_Dakota','Tennessee','Texas','Utah','Vermont','Virginia','Washington','West_Virginia','Wisconsin','Wyoming');

-- CREATE TYPE us_territories AS ENUM ('American_Samoa', 'Guam', 'Northern_Mariana_Islands', 'Puerto_Rico', 'Virgin_Islands', 'District_of_Columbia');

-- CREATE TYPE user_type AS ENUM (
--        'guest',
--        'regular',
--        'subadmin',
--        'admin'
-- );

-- CREATE TABLE items (id bigserial PRIMARY KEY, embedding vector(384));

CREATE TABLE IF NOT EXISTS accounts (
        account_id SERIAL PRIMARY KEY,
        account_slug TEXT NOT NULL DEFAULT (uuid_generate_v4()),
        account_name TEXT NOT NULL UNIQUE,
        account_secret TEXT DEFAULT NULL,
        created_at TIMESTAMPTZ DEFAULT NOW(),
        updated_at TIMESTAMPTZ DEFAULT NULL
    );

CREATE TABLE IF NOT EXISTS users (
        user_id SERIAL PRIMARY KEY,
        user_slug TEXT NOT NULL DEFAULT (uuid_generate_v4()),
        account_id INTEGER NOT NULL DEFAULT 3,
        username TEXT NOT NULL UNIQUE,
        email TEXT NOT NULL UNIQUE,
        user_type_id INT NOT NULL DEFAULT 2,
        secret TEXT DEFAULT NULL,
        password TEXT NOT NULL,

        user_subs INTEGER[] DEFAULT ARRAY[1]::INTEGER[],
        location_subs INTEGER[] DEFAULT ARRAY[]::INTEGER[],

        avatar_path TEXT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ DEFAULT NULL,
        CONSTRAINT fk_account_id
            FOREIGN KEY(account_id) 
	            REFERENCES accounts(account_id)
    );

CREATE TABLE IF NOT EXISTS user_details (
        user_details_id SERIAL PRIMARY KEY,
        user_id INTEGER NOT NULL,
        address_one TEXT NOT NULL,
        address_two TEXT NULL,
        city TEXT NOT NULL,
        state CHAR(2) NOT NULL,
        zip VARCHAR (5) NOT NULL,
        dob DATE NOT NULL,
        primary_phone TEXT NOT NULL,
        mobile_phone TEXT NULL,
        secondary_phone TEXT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ DEFAULT NULL,
        CONSTRAINT fk_user_id
            FOREIGN KEY(user_id) 
	            REFERENCES users(user_id)
    );

CREATE TABLE IF NOT EXISTS user_settings (
        user_settings_id SERIAL PRIMARY KEY,
        user_id INTEGER NOT NULL,
        theme_id INTEGER NOT NULL DEFAULT 1,
        list_view TEXT NOT NULL DEFAULT 'consult',
        notifications BOOLEAN NOT NULL DEFAULT FALSE,
        newsletter BOOLEAN NOT NULL DEFAULT FALSE,
        updated_at TIMESTAMPTZ DEFAULT NULL,
        CONSTRAINT fk_user_id
            FOREIGN KEY(user_id) 
	            REFERENCES users(user_id)
    );

CREATE TABLE IF NOT EXISTS user_sessions (
        user_session_id SERIAL PRIMARY KEY,
        user_id INTEGER NOT NULL,
        session_id TEXT NOT NULL,
        -- session_id TEXT NOT NULL DEFAULT (uuid_generate_v4()),
        expires TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        logout BOOLEAN NOT NULL DEFAULT FALSE,
        created_at TIMESTAMPTZ DEFAULT NOW(),
        updated_at TIMESTAMPTZ DEFAULT NULL,
        CONSTRAINT fk_user_id
            FOREIGN KEY(user_id) 
	            REFERENCES users(user_id)
    );

CREATE TABLE IF NOT EXISTS reset_password_requests (
        request_id SERIAL PRIMARY KEY,
        user_id INTEGER NULL,
        req_ip TEXT NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        CONSTRAINT fk_user_id
            FOREIGN KEY(user_id) 
	            REFERENCES users(user_id)
    );

CREATE TABLE IF NOT EXISTS specialties (
        specialty_id SERIAL PRIMARY KEY,
        specialty_name TEXT NOT NULL
    );

CREATE TABLE IF NOT EXISTS states (
        state_id SERIAL PRIMARY KEY,
        state_name CHAR(2) NOT NULL
    );

INSERT INTO states (state_name)
    VALUES ('AL'),('AK'),('AZ'),('AR'),('CA'),('CO'),('CT'),('DE'),('FL'),('GA'),('HI'),('ID'),('IL'),('IN'),('IA'),('KS'),('KY'),('LA'),('ME'),('MD'),('MA'),
        ('MI'),('MN'),('MS'),('MO'),('MT'),('NE'),('NV'),('NH'),('NJ'),('NM'),('NY'),('NC'),('ND'),('OH'),('OK'),('OR'),('PA'),('RI'),('SC'),('SD'),('TN'),
        ('TX'),('UT'),('VT'),('VA'),('WA'),('WV'),('WI'),('WY'),('AS'),('GU'),('MP'),('PR'),('VI'),('DC');

CREATE TABLE IF NOT EXISTS entities (
        entity_id SERIAL PRIMARY KEY,
        entity_name TEXT NOT NULL
    );

CREATE TABLE IF NOT EXISTS loan_purpose (
        loan_purpose_id SERIAL PRIMARY KEY,
        loan_purpose_name TEXT NOT NULL
    );

CREATE TABLE IF NOT EXISTS mime_types (
        mime_type_id SERIAL PRIMARY KEY,
        mime_type_name TEXT NOT NULL
    );

CREATE TABLE IF NOT EXISTS user_types (
        user_type_id SERIAL PRIMARY KEY,
        user_type_name TEXT NOT NULL UNIQUE
    );

CREATE TABLE IF NOT EXISTS article_categories (
        category_id SERIAL PRIMARY KEY,
        category_name TEXT NOT NULL UNIQUE
    );


CREATE TABLE IF NOT EXISTS territories (
        territory_id SERIAL PRIMARY KEY,
        territory_name TEXT NOT NULL,
        territory_states TEXT[] NULL
    );

CREATE TABLE IF NOT EXISTS contacts (
    contact_id SERIAL PRIMARY KEY,
    contact_title TEXT NULL,
    contact_f_name TEXT NOT NULL,
    contact_l_name TEXT NULL,
    contact_email TEXT NOT NULL,
    contact_primary_phone TEXT NULL,
    contact_secondary_phone TEXT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NULL
);

-- FIXME: Add PostGIS and lat/long
CREATE TABLE IF NOT EXISTS locations (
        location_id SERIAL PRIMARY KEY,
        location_slug TEXT NOT NULL DEFAULT (uuid_generate_v4()),
        location_name TEXT NOT NULL,
        location_address_one TEXT NOT NULL,
        location_address_two TEXT NULL,
        location_city TEXT NOT NULL,
        location_state CHAR(2) NOT NULL,
        location_zip VARCHAR (5) NOT NULL,
        location_phone TEXT NULL,
        location_contact_id INTEGER NOT NULL DEFAULT 1,
        territory_id INTEGER NOT NULL DEFAULT 1,
        created_at TIMESTAMPTZ DEFAULT NOW(),
        updated_at TIMESTAMPTZ DEFAULT NULL,
        CONSTRAINT fk_contact
            FOREIGN KEY(location_contact_id) 
	            REFERENCES contacts(contact_id),
        CONSTRAINT fk_territory
            FOREIGN KEY(territory_id) 
	            REFERENCES territories(territory_id)
    );

CREATE TABLE IF NOT EXISTS applications (
        application_id SERIAL PRIMARY KEY,
        application_slug TEXT NOT NULL DEFAULT (uuid_generate_v4()),
        location_id INTEGER NOT NULL,
        first_name TEXT NOT NULL,
        last_name TEXT NOT NULL,
        -- user_id INTEGER NOT NULL,
        address_one TEXT NOT NULL, 
        address_two TEXT NULL, 
        city TEXT NOT NULL, 
        state CHAR(2), 
        zip CHAR(5), 
        phone CHAR(14), 
        ssn_nacl TEXT NOT NULL, 
        dob DATE NULL,
        annual_income INTEGER NOT NULL,
        marital_status INTEGER NOT NULL, 
        desired_loan_amount INTEGER NOT NULL, 
        loan_purpose INTEGER NOT NULL, 
        homeownership INTEGER NOT NULL, 
        employment_status INTEGER NOT NULL, 
        emp_length INTEGER NOT NULL, 
        created_at TIMESTAMPTZ DEFAULT NOW(),
        updated_at TIMESTAMPTZ DEFAULT NULL,
        -- CONSTRAINT fk_user
        --     FOREIGN KEY(user_id) 
	    --         REFERENCES users(user_id),
        CONSTRAINT fk_loan_purpose
            FOREIGN KEY(loan_purpose) 
	            REFERENCES loan_purpose(loan_purpose_id),
        CONSTRAINT fk_location
            FOREIGN KEY(location_id) 
	            REFERENCES locations(location_id)
    );

CREATE TABLE IF NOT EXISTS servicers (
        servicer_id SERIAL PRIMARY KEY,
        servicer_slug TEXT NOT NULL DEFAULT (uuid_generate_v4()),
        servicer_name TEXT NOT NULL,
        contact_name TEXT NULL,
        contact_phone TEXT NULL,
        created_at TIMESTAMPTZ DEFAULT NOW(),
        updated_at TIMESTAMPTZ DEFAULT NULL
    );

CREATE TABLE IF NOT EXISTS offers (
        offer_id SERIAL PRIMARY KEY,
        offer_slug TEXT NOT NULL DEFAULT (uuid_generate_v4()),
        application_id INTEGER NOT NULL,
        servicer_id INTEGER NOT NULL,
        max_amount INTEGER NOT NULL,
        min_amount INTEGER NOT NULL,
        terms INTEGER NOT NULL,
        apr REAL NOT NULL,
        percent_fee REAL NOT NULL,
        expires DATE NOT NULL,
        -- changesets JSONB DEFAULT NULL,
        created_at TIMESTAMPTZ DEFAULT NOW(),
        updated_at TIMESTAMPTZ DEFAULT NULL,
        CONSTRAINT fk_application
            FOREIGN KEY(application_id) 
	            REFERENCES applications(application_id),
        CONSTRAINT fk_services
            FOREIGN KEY(servicer_id) 
	            REFERENCES servicers(servicer_id)
        -- FIXME: Index
    );

CREATE TABLE IF NOT EXISTS borrowers (
        borrower_id SERIAL PRIMARY KEY,
        borrower_slug TEXT NOT NULL DEFAULT (uuid_generate_v4()),
        location_id INTEGER NOT NULL DEFAULT 3,
        f_name  TEXT NOT NULL,
        l_name  TEXT NOT NULL,
        dob DATE NOT NULL,
        email TEXT NOT NULL UNIQUE,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ DEFAULT NULL,
        CONSTRAINT fk_location_id
            FOREIGN KEY(location_id) 
	            REFERENCES locations(location_id)
    );

CREATE TABLE IF NOT EXISTS credit_file (
        borrower_id INTEGER NOT NULL,
        emp_title TEXT NOT NULL,
        emp_length INTEGER NOT NULL DEFAULT 0,
        state CHAR(2) NOT NULL,
        homeownership INTEGER NOT NULL DEFAULT 0,
        annual_income INTEGER NOT NULL DEFAULT 0.0,
        verified_income INTEGER NOT NULL DEFAULT 1,
        debt_to_income REAL NOT NULL DEFAULT 0.0,
        annual_income_joint INTEGER NULL,
        verification_income_joint INTEGER NULL,
        debt_to_income_joint REAL NULL,
        delinq_2y INTEGER NOT NULL DEFAULT 0,
        months_since_last_delinq INTEGER NULL,
        earliest_credit_line INTEGER NOT NULL,
        inquiries_last_12m INTEGER NOT NULL DEFAULT 0,
        total_credit_lines INTEGER NOT NULL DEFAULT 0,
        open_credit_lines INTEGER NOT NULL DEFAULT 0,
        total_credit_limit INTEGER NOT NULL DEFAULT 0,
        total_credit_utilized INTEGER NOT NULL DEFAULT 0,
        num_collections_last_12m REAL NOT NULL DEFAULT 0.0,
        num_historical_failed_to_pay REAL NOT NULL DEFAULT 0.0,
        months_since_90d_late INTEGER NULL,
        current_accounts_delinq INTEGER NOT NULL DEFAULT 0,		
        total_collection_amount_ever INTEGER NOT NULL DEFAULT 0,		
        current_installment_accounts INTEGER NOT NULL DEFAULT 0,		
        accounts_opened_24m INTEGER NOT NULL DEFAULT 0,	
        months_since_last_credit_inquiry INTEGER NULL,	
        num_satisfactory_accounts INTEGER NOT NULL DEFAULT 0,	
        num_accounts_120d_past_due INTEGER NOT NULL DEFAULT 0,	
        num_accounts_30d_past_due INTEGER NOT NULL DEFAULT 0,	
        num_active_debit_accounts INTEGER NOT NULL DEFAULT 0,	
        total_debit_limit INTEGER NOT NULL DEFAULT 0,	
        num_total_cc_accounts INTEGER NOT NULL DEFAULT 0,
        num_open_cc_accounts INTEGER NOT NULL DEFAULT 0,
        num_cc_carrying_balance INTEGER NOT NULL DEFAULT 0,	
        num_mort_accounts INTEGER NOT NULL DEFAULT 0,	
        account_never_delinq_percent REAL NOT NULL DEFAULT 100.0,
        tax_liens INTEGER NOT NULL DEFAULT 0,
        public_record_bankrupt INTEGER NOT NULL DEFAULT 0,
        CONSTRAINT fk_borrower
            FOREIGN KEY(borrower_id) 
                REFERENCES borrowers(borrower_id)
        -- FIXME: Index
    );

CREATE TABLE IF NOT EXISTS loans (
        loan_id SERIAL PRIMARY KEY,
        borrower_id INTEGER NOT NULL,
        application_id INTEGER NOT NULL,
        servicer_id INTEGER NOT NULL,
        loan_purpose INTEGER NOT NULL DEFAULT 0,
        application_type INTEGER NOT NULL DEFAULT 0,
        loan_amount INTEGER NOT NULL DEFAULT 0,
        term INTEGER NOT NULL DEFAULT 0,
        interest_rate REAL NOT NULL DEFAULT 0.0,
        installment REAL NOT NULL DEFAULT 0.0,
        grade CHAR(1) NOT NULL,
        sub_grade CHAR(2) NOT NULL,
        issue_month CHAR(8) NOT NULL,
        loan_status INTEGER NOT NULL DEFAULT 0,
        initial_listing_status INTEGER NOT NULL DEFAULT 0,
        disbursement_method INTEGER NOT NULL DEFAULT 0,
        balance REAL NOT NULL DEFAULT 0.0,
        paid_total REAL NOT NULL DEFAULT 0.0,
        paid_principal REAL NOT NULL DEFAULT 0.0,
        paid_interest REAL NOT NULL DEFAULT 0.0,
        paid_late_fees REAL NOT NULL DEFAULT 0.0,
        CONSTRAINT fk_application
            FOREIGN KEY(application_id) 
	            REFERENCES applications(application_id),
        CONSTRAINT fk_borrower
            FOREIGN KEY(borrower_id) 
	            REFERENCES borrowers(borrower_id),
        CONSTRAINT fk_servicer
            FOREIGN KEY(servicer_id) 
	            REFERENCES servicers(servicer_id)
    );

CREATE TABLE IF NOT EXISTS messages (
        message_id SERIAL PRIMARY KEY,
        content TEXT NOT NULL,
        subject TEXT NOT NULL,
        sent_to INTEGER NOT NULL,
        sent_from INTEGER NOT NULL,
        created_at TIMESTAMPTZ DEFAULT NOW(),
        updated_at TIMESTAMPTZ DEFAULT NOW(),
        sent_at TIMESTAMPTZ DEFAULT NULL,
        read_at TIMESTAMPTZ DEFAULT NULL,
        CONSTRAINT fk_sent_to
            FOREIGN KEY(sent_to) 
	            REFERENCES users(user_id),
        CONSTRAINT fk_sent_from
            FOREIGN KEY(sent_from) 
	            REFERENCES users(user_id)
    );

CREATE TABLE IF NOT EXISTS attachments (
        attachment_id SERIAL PRIMARY KEY,
        path TEXT UNIQUE NOT NULL,
        user_id INTEGER NOT NULL,
        -- mime_type mime_type NOT NULL,
        -- channel attachment_channel NOT NULL,
        mime_type_id INTEGER NOT NULL,
        channel TEXT NOT NULL,
        short_desc TEXT NOT NULL DEFAULT 'No Description',
        created_at TIMESTAMPTZ DEFAULT NOW(),
        updated_at TIMESTAMPTZ DEFAULT NOW(),
        CONSTRAINT fk_user_id
            FOREIGN KEY(user_id) 
	            REFERENCES users(user_id)
    );

INSERT INTO territories (territory_id, territory_name, territory_states)
VALUES
(1, 'national',     NULL),
(2, 'northeast',    ARRAY['DE', 'MD', 'PA', 'NJ', 'NY', 'MA', 'CT', 'VT', 'NH', 'RI', 'ME', 'OH']),
(3, 'southeast',    ARRAY['AR', 'LA', 'MS', 'TN', 'AL', 'KY', 'WV', 'VA', 'NC', 'SC', 'GA', 'FL']),
(4, 'west',         ARRAY['CA', 'WA', 'OR', 'NV', 'AZ', 'NM', 'UT', 'WY', 'ID', 'MT', 'AK', 'CO', 'WY']),
(5, 'midwest',      ARRAY['NE', 'IA', 'KS', 'OK', 'MO', 'SD', 'ND', 'MN', 'WI', 'MI', 'IN', 'IL', 'TX']);

INSERT INTO specialties (specialty_id, specialty_name)
VALUES
(1, 'general'),
(2, 'finance'),
(3, 'insurance'),
(4, 'technology'),
(5, 'government'),
(6, 'legal');

INSERT INTO user_types (user_type_id, user_type_name)
VALUES
(1, 'admin'),
(2, 'subadmin'),
(3, 'regular'),
(4, 'guest');

INSERT INTO entities (entity_id, entity_name)
VALUES
(1, 'user'),
(2, 'admin'),
(3, 'subadmin'),
(4, 'consultant'),
(5, 'location'),
(6, 'consult'),
(7, 'client'),
(8, 'query');

INSERT INTO loan_purpose (loan_purpose_id, loan_purpose_name)
VALUES
(1, 'debt_consolidation'),
(2, 'medical'),
(3, 'house'),
(4, 'car'),
(5, 'education'),
(6, 'other');

INSERT INTO article_categories (category_id, category_name)
VALUES
(1, 'general'),
(2, 'team-building'),
(3, 'strategy'),
(4, 'company-alert');

INSERT INTO mime_types (mime_type_id, mime_type_name)
VALUES
(1, 'image/png'),
(2, 'image/jpeg'),
(3, 'image/gif'),
(4, 'image/webp'),
(5, 'image/svg+xml'),
(6, 'audio/wav'),
(7, 'audio/mpeg'),
(8, 'audio/webm'),
(9, 'video/webm'),
(10, 'video/mpeg'),
(11, 'video/mp4'),
(12, 'application/json'),
(13, 'application/pdf'),
(14, 'text/csv'),
(15, 'text/html'),
(16, 'text/calendar');

INSERT INTO accounts (account_name, account_secret) 
VALUES 
('root',                    'root_secret'),
('admin',                   'admin_secret'),
('default_user',            'user_secret'),
('default_client',          'client_secret'),
('default_company_client',  'company_client_secret');

INSERT INTO users (username, location_subs, user_type_id, account_id, email, password) 
VALUES 
('root',                DEFAULT,    1, 1, 'root@consultancy.com',           '$argon2id$v=19$m=19456,t=2,p=1$3H44ziOiAHHPL3u5x+S+Ag$YowYSA614EokasKaa5BCx+2Dtmyf+53HE+LB3EinfiQ'),
('admin',               DEFAULT,    1, 2, 'admin@consultancy.com',          '$argon2id$v=19$m=19456,t=2,p=1$3H44ziOiAHHPL3u5x+S+Ag$YowYSA614EokasKaa5BCx+2Dtmyf+53HE+LB3EinfiQ'),
-- Users
('jim_jam',             DEFAULT,    3, 2, 'jim@jam.com',                    '$argon2id$v=19$m=19456,t=2,p=1$3H44ziOiAHHPL3u5x+S+Ag$YowYSA614EokasKaa5BCx+2Dtmyf+53HE+LB3EinfiQ'),
('aaron',               ARRAY[7],   3, 2, 'aaron@aaron.com',                '$argon2id$v=19$m=19456,t=2,p=1$3H44ziOiAHHPL3u5x+S+Ag$YowYSA614EokasKaa5BCx+2Dtmyf+53HE+LB3EinfiQ'),
-- Clients
('first_client',        DEFAULT,    3, 3, 'client_one@client.com',            '$argon2id$v=19$m=19456,t=2,p=1$3H44ziOiAHHPL3u5x+S+Ag$YowYSA614EokasKaa5BCx+2Dtmyf+53HE+LB3EinfiQ'),
('second_client',       DEFAULT,    3, 3, 'client_two@client.com',            '$argon2id$v=19$m=19456,t=2,p=1$3H44ziOiAHHPL3u5x+S+Ag$YowYSA614EokasKaa5BCx+2Dtmyf+53HE+LB3EinfiQ'),
-- Subadmins
('sudadmin_one',        DEFAULT,    2, 3, 'subadmin_one@subadmin.com',         '$argon2id$v=19$m=19456,t=2,p=1$3H44ziOiAHHPL3u5x+S+Ag$YowYSA614EokasKaa5BCx+2Dtmyf+53HE+LB3EinfiQ'),
-- Consultants
('hulk_hogan',          DEFAULT,    2, 2, 'hulk_hogan@consultancy.com',        '$argon2id$v=19$m=19456,t=2,p=1$3H44ziOiAHHPL3u5x+S+Ag$YowYSA614EokasKaa5BCx+2Dtmyf+53HE+LB3EinfiQ'),
('mike_ryan',           DEFAULT,    2, 2, 'mike_ryan@consultancy.com',         '$argon2id$v=19$m=19456,t=2,p=1$3H44ziOiAHHPL3u5x+S+Ag$YowYSA614EokasKaa5BCx+2Dtmyf+53HE+LB3EinfiQ'),
('zardos',              DEFAULT,    2, 2, 'zardos@consultancy.com',            '$argon2id$v=19$m=19456,t=2,p=1$3H44ziOiAHHPL3u5x+S+Ag$YowYSA614EokasKaa5BCx+2Dtmyf+53HE+LB3EinfiQ'), -- 10
('gregs_lobos',         DEFAULT,    2, 2, 'gregs_lobos@consultancy.com',       '$argon2id$v=19$m=19456,t=2,p=1$3H44ziOiAHHPL3u5x+S+Ag$YowYSA614EokasKaa5BCx+2Dtmyf+53HE+LB3EinfiQ'),
('rob_bower',           DEFAULT,    2, 2, 'rob_bower@consultancy.com',         '$argon2id$v=19$m=19456,t=2,p=1$3H44ziOiAHHPL3u5x+S+Ag$YowYSA614EokasKaa5BCx+2Dtmyf+53HE+LB3EinfiQ'),
('v_smith',             DEFAULT,    2, 2, 'v_smith@consultancy.com',           '$argon2id$v=19$m=19456,t=2,p=1$3H44ziOiAHHPL3u5x+S+Ag$YowYSA614EokasKaa5BCx+2Dtmyf+53HE+LB3EinfiQ'),
('joe_z',               DEFAULT,    2, 2, 'joe_z@consultancy.com',             '$argon2id$v=19$m=19456,t=2,p=1$3H44ziOiAHHPL3u5x+S+Ag$YowYSA614EokasKaa5BCx+2Dtmyf+53HE+LB3EinfiQ'),

('to_be_consultant',    DEFAULT,    3, 2, 'to_be_c@consultancy.com',           '$argon2id$v=19$m=19456,t=2,p=1$3H44ziOiAHHPL3u5x+S+Ag$YowYSA614EokasKaa5BCx+2Dtmyf+53HE+LB3EinfiQ'),

('jamie',               DEFAULT,    2, 2, 'jamie@consultancy.com',              '$argon2id$v=19$m=19456,t=2,p=1$3H44ziOiAHHPL3u5x+S+Ag$YowYSA614EokasKaa5BCx+2Dtmyf+53HE+LB3EinfiQ'), -- 16
('alexandra',           DEFAULT,    2, 2, 'alexandra@consultancy.com',          '$argon2id$v=19$m=19456,t=2,p=1$3H44ziOiAHHPL3u5x+S+Ag$YowYSA614EokasKaa5BCx+2Dtmyf+53HE+LB3EinfiQ'),
('jimcoats',            DEFAULT,    2, 2, 'jimcoats@consultancy.com',           '$argon2id$v=19$m=19456,t=2,p=1$3H44ziOiAHHPL3u5x+S+Ag$YowYSA614EokasKaa5BCx+2Dtmyf+53HE+LB3EinfiQ'),
('kevink',              DEFAULT,    2, 2, 'kevink@consultancy.com',             '$argon2id$v=19$m=19456,t=2,p=1$3H44ziOiAHHPL3u5x+S+Ag$YowYSA614EokasKaa5BCx+2Dtmyf+53HE+LB3EinfiQ'),
('jennifer',            DEFAULT,    2, 2, 'jennifer@consultancy.com',           '$argon2id$v=19$m=19456,t=2,p=1$3H44ziOiAHHPL3u5x+S+Ag$YowYSA614EokasKaa5BCx+2Dtmyf+53HE+LB3EinfiQ'),
('luke',                DEFAULT,    2, 2, 'luke@consultancy.com',               '$argon2id$v=19$m=19456,t=2,p=1$3H44ziOiAHHPL3u5x+S+Ag$YowYSA614EokasKaa5BCx+2Dtmyf+53HE+LB3EinfiQ');

INSERT INTO user_details (user_id, address_one, address_two, city, state, zip, dob, primary_phone) 
VALUES 
(7, '12 Subadmin Dr', NULL, 'Omaha', 'NE', '68124', '1980-01-05', '402-333-3333'),
(8, '12 Subadmin Dr', NULL, 'Omaha', 'NE', '68124', '1980-01-05', '402-333-3333'),
(9, '12 Subadmin Dr', NULL, 'Omaha', 'NE', '68124', '1980-01-05', '402-333-3333'),
(10, '12 Subadmin Dr', NULL, 'Omaha', 'NE', '68124', '1980-01-05', '402-333-3333'),
(11, '12 Subadmin Dr', NULL, 'Omaha', 'NE', '68124', '1980-01-05', '402-333-3333'),
(12, '12 Subadmin Dr', NULL, 'Omaha', 'NE', '68124', '1980-01-05', '402-333-3333'),
(13, '12 Subadmin Dr', NULL, 'Omaha', 'NE', '68124', '1980-01-05', '402-333-3333'),
(14, '12 Subadmin Dr', NULL, 'Omaha', 'NE', '68124', '1980-01-05', '402-333-3333');

INSERT INTO user_settings (user_id, theme_id, list_view) 
VALUES 
(1, 1, DEFAULT),
(2, 1, DEFAULT),
(3, 1, DEFAULT),
(4, 1, DEFAULT),
(5, 1, DEFAULT),
(6, 1, DEFAULT),
(7, 1, DEFAULT),
(8, 1, 'consultant'),
(9, 1, 'consultant'),
(10, 1, 'consultant'),
(11, 1, 'consultant'),
(12, 1, 'consultant'),
(13, 1, 'consultant'),
(14, 1, 'consultant'),
(15, 1, DEFAULT);

INSERT INTO user_sessions (user_id, session_id, expires, created_at) 
VALUES 
(1, 'c4689973-82eb-404a-a249-e684cadb31df', NOW() - '20 days'::interval, NOW() - '21 days'::interval),
(1, '7d9527cb-44e5-4f2d-813f-6d2ed5ed92a2', NOW() - '15 days'::interval, NOW() - '16 days'::interval),
(2, 'cb8984a0-d6cb-4f4c-8dc2-0209c5b5f027', NOW() - '14 days'::interval, NOW() - '15 days'::interval);

INSERT INTO contacts (contact_title, contact_f_name, contact_l_name, contact_email, contact_primary_phone, contact_secondary_phone) 
VALUES 
('Site Admin',       'Greg',  'Cote',   'cote@gregslobos.com',  '555-555-5555', '555-555-5555'),
('Location Manager', 'Billy', 'Gil',    'bill@marlins.com',     '555-555-5555', '555-555-5555');

INSERT INTO locations (location_name, location_address_one, location_address_two, location_city, location_state, location_zip, location_phone, location_contact_id, territory_id) 
VALUES 
('Default - Main Office',   '1234 Main St.',        NULL,       'Omaha',                            'NE', '68114', '555-555-5555', DEFAULT, 5),
('Bend Conference Center',  '5432 Postgres Ave',    'Ste. 101', 'Bend',                             'OR', '97701', '555-555-5555', DEFAULT, 4),
('101 W',                   '101 W. Ave',           'Ste. 901', 'Chicago',                          'IL', '60007', '555-555-5555', DEFAULT, 5),
('Hilton New York',         '1001 Western St.',     NULL,       'New York',                         'NY', '10001', '555-555-5555', DEFAULT, 2),
('Islands Local',           '70 Oahu Ave',          'Pt. 12',   'Honolulu',                         'HI', '96805', '555-555-5555', DEFAULT, 4),
('LAX Sidepost',            '1 World Way',          NULL,       'Los Angeles',                      'CA', '90045', '555-555-5555', DEFAULT, 4),
('Grosse Pointe Main',      '1212 Main Ln.',        NULL,       'Village of Grosse Pointe Shores',  'MI', '48236', '555-555-5555', DEFAULT, 5),
('Austin Heights',          '6379 Redis Lane',      NULL,       'Austin',                           'TX', '78799', '555-555-5555', 2,       5),
('Principal Arena',         '98 Santana Ave',       'Ofc. 2',   'Rapid City',                       'SD', '57701', '555-555-5555', DEFAULT, 5),
('New Balam Home',          '801 Haliburton Dr.',   'Ste. 101', 'Phoenix',                          'AZ', '85007', '555-555-5555', DEFAULT, 4),  -- 10
('McGillicuddy & Sons',     '300 South Beach Dr.',  NULL,       'Miami',                            'FL', '33109', '555-555-5555', DEFAULT, 3),
('Boston Ceremonial',       '7878 Paul Revere St.', NULL,       'Boston',                           'MA', '02117', '555-555-5555', DEFAULT, 2),
('The Machine Shed',        '1674 Grant St.',       NULL,       'Des Moines',                       'IA', '96805', '555-555-5555', DEFAULT, 5),
('Big Little Building',     '1 Luca Ave',           NULL,       'Reno',                             'NV', '90045', '555-555-5555', DEFAULT, 4),
('The ATL Sky',             '1212 Main Ln.',        NULL,       'Atlanta',                          'GA', '48236', '555-555-5555', DEFAULT, 3),
('Meyer Home',              '771 Benny Dr.',        NULL,       'Dallas',                           'TX', '75001', '555-555-5555', DEFAULT, 5),
('Patton & Smoler',         '0909 Smith Road',      NULL,       'Olympia',                          'WA', '98506', '555-555-5555', DEFAULT, 4),
('Mudra International',     '7878 Homewater St.',   NULL,       'Edina',                            'MN', '55343', '555-555-5555', DEFAULT, 5),
('St. Olaf College',        '1500 St. Olaf Ave.',   NULL,       'Northfield',                       'MN', '55057', '555-555-5555', DEFAULT, 5),
('National Location #1',    '101 National Dr.',     NULL,       'Kansas City',                      'MO', '64109', '555-555-5555', DEFAULT, DEFAULT),  -- 20
('Thompson Palace',         '1 Mesmer Ave',         'Ste. 222', 'Philadelphia',                     'PA', '19099', '555-555-5555', DEFAULT, 2),
('NOLA Center',             '434 Main Dr.',         NULL,       'New Orleans',                      'LA', '70115', '555-555-5555', DEFAULT, 3),
('MP Heights',              '09 Hermes Way',        NULL,       'Montpelier',                       'VT', '05604', '555-555-5555', 2,       2);
-- audio/flac
INSERT INTO attachments (path, mime_type_id, user_id, channel, short_desc, created_at) 
VALUES 
('https://upload.wikimedia.org/wikipedia/commons/5/5d/Kuchnia_polska-p243b.png',            1,  3, 'Upload', 'Polska PNG',      '2023-09-11 19:10:25-06'),
('https://upload.wikimedia.org/wikipedia/commons/3/3f/Rail_tickets_of_Poland.jpg',          2,  3, 'Upload', 'Polska JPG',      '2023-09-11 19:10:25-06'),
('https://upload.wikimedia.org/wikipedia/commons/f/f4/Larynx-HiFi-GAN_speech_sample.wav',   6,  3, 'Upload', 'Polska WAV',      '2023-09-11 19:10:25-06'),
('https://upload.wikimedia.org/wikipedia/commons/6/6e/Mindannyian-vagyunk.webm',            9,  3, 'Upload', 'Polska WEBM',     '2023-09-14 19:16:25-06'),
('https://upload.wikimedia.org/wikipedia/commons/f/f5/Kuchnia_polska-p35b.png',             1,  4, 'Email',  'Polska PNG #2',   '2023-09-16 16:00:25-06'),
('https://upload.wikimedia.org/wikipedia/commons/d/d7/Programmation_Ruby-fr.pdf',           13, 4, 'Email',  'Ruby PDF',        '2023-10-16 16:00:25-06'),
('https://upload.wikimedia.org/wikipedia/commons/b/b4/Apache.pdf',                          13, 3, 'Upload', 'Apache PDF',      '2023-09-18 19:16:25-06'),
('https://upload.wikimedia.org/wikipedia/commons/5/50/Greenscreen_Computer_Animation.mpg',  10, 2, 'Upload', 'Mpg Video',       '2023-10-11 14:16:25-06'),
('https://upload.wikimedia.org/wikipedia/commons/0/01/Do_%281%29.mp3',                      7,  2, 'Upload', 'MP3 Audio(Mpeg)', '2023-10-12 13:15:55-06'),
('/media/cities.csv',                                                                       14, 2, 'Upload', 'CSV File #1',     '2023-11-12 13:15:55-06');

INSERT INTO applications (application_slug, location_id, first_name, last_name, address_one, address_two, city, state, zip, annual_income, phone, dob, desired_loan_amount, loan_purpose, marital_status, ssn_nacl, homeownership, employment_status, emp_length) 
VALUES 
('1dff7c61-98ed-486a-9bc1-0f323268199d', 1, 'Tim', 'Jones','7724 Pine Cir', NULL, 'Omaha', 'NE', 68124, 75000,'555-555-5555', '1987-01-01', 65000, 2, 1, DIGEST('000000000', 'sha256'), 1, 1, 5),
('522a3933-7d69-4b38-8840-1ae10945094b', 1, 'Steve', 'Louise','4483 South 87th', NULL, 'Northfield', 'MN', 68124, 88000, '555-555-5555', '1976-01-01', 65000, 2, 1, DIGEST('000000000', 'sha256'), 1, 1, 5);

INSERT INTO servicers (servicer_name, contact_name, contact_phone) 
VALUES 
('Lending  Club', 'Sreeni Yachtface', NULL),
('FirstSource', 'Jenny Smith', '333-333-3333'),
('MainLoan', 'Kevin Griswald', NULL),
('WonderCare', 'Greg Cote', NULL),
('National Loans', 'Jim Jones', '555-555-5555');

INSERT INTO borrowers (location_id, f_name, l_name, dob, email) 
VALUES 
(1, 'Jen', 'Smith', '1987-04-01', 'b1@eg.com'),
(1, 'Chris', 'Reynolds', '1980-08-11', 'b2@eg.com'),
(1, 'Eric', 'Jackson', '1975-09-21', 'b3@eg.com'),
(2, 'Rick', 'Garcia', '1956-03-30', 'b4@eg.com'),
(2, 'Ellen', 'Lobos', '1965-12-19', 'b5@eg.com'),
(2, 'Louise', 'Wilson', '1971-11-09', 'b6@eg.com'),
(1, 'George', 'Ambers', '1988-09-03', 'b7@eg.com'),
(2, 'Rob', 'Lucas', '1992-03-08', 'b8@eg.com'),
(1, 'Jake', 'Kielen', '1973-05-28', 'b9@eg.com');

INSERT INTO offers (servicer_id, application_id, max_amount, min_amount, terms, apr, percent_fee, expires) 
VALUES 
(1, 1, 15000, 5000, 36, 12.99, 3.2, DATE(NOW() + INTERVAL '4 week')),
(2, 2, 17000, 5000, 36, 15.99, 2.2, DATE(NOW() + INTERVAL '2 week'));

INSERT INTO credit_file (borrower_id, emp_title, emp_length, state, homeownership, annual_income, verified_income,debt_to_income,annual_income_joint, verification_income_joint, debt_to_income_joint,
        delinq_2y, months_since_last_delinq, earliest_credit_line, inquiries_last_12m, total_credit_lines, open_credit_lines, total_credit_limit, total_credit_utilized, 
        num_collections_last_12m, num_historical_failed_to_pay, months_since_90d_late, current_accounts_delinq, total_collection_amount_ever, current_installment_accounts, 
        accounts_opened_24m, months_since_last_credit_inquiry, num_satisfactory_accounts, num_accounts_120d_past_due, num_accounts_30d_past_due, num_active_debit_accounts, 
        total_debit_limit, num_total_cc_accounts, num_open_cc_accounts, num_cc_carrying_balance, num_mort_accounts, account_never_delinq_percent, tax_liens, public_record_bankrupt)
VALUES 
(1, 'president',              3, 'NE', 1, 90000, 1, 18.01, NULL, NULL, NULL, 0, NULL, 2001, 6, 28, 10, 70795, 38767, 0, 0, NULL, 0, 0, 2, 5, NULL, 0, 0, 0, 2, 11100, 14, 8, 6, 1, 92.9, 0, 0),
(2, 'sales representative',   7, 'MN', 2, 45000, 1, 18.01, NULL, NULL, NULL, 0, NULL, 2001, 6, 28, 10, 40795, 22767, 0, 0, NULL, 0, 0, 2, 5, NULL, 0, 0, 0, 2, 11100, 14, 8, 6, 1, 92.9, 0, 0);

INSERT INTO loans (borrower_id, application_id, servicer_id, loan_purpose, application_type, loan_amount, term, interest_rate, installment, grade, sub_grade,
                    issue_month, loan_status, initial_listing_status, disbursement_method, balance, paid_total, paid_principal, paid_interest, paid_late_fees)
VALUES 
(1, 1, 1, 2, 1, 28000, 60, 14.07, 652.53, 'C', 'C3', 'Mar-2018', 1, 2, 1, 27015.86, 1999.33, 984.14, 1015.19, 0),
(2, 2, 1, 2, 1, 18000, 60, 14.07, 452.53, 'A', 'A2', 'Jun-2018', 1, 2, 1, 21335.86, 1299.33, 984.14, 1015.19, 0);



-- Triggers

CREATE OR REPLACE FUNCTION user_settings_insert_trigger_fnc()
  RETURNS trigger AS
$$
BEGIN
    INSERT INTO "user_settings" ( "user_settings_id", "user_id" )
         VALUES(DEFAULT, NEW."user_id");
RETURN NEW;
END;
$$
LANGUAGE 'plpgsql';

CREATE TRIGGER user_settings_insert_trigger
  AFTER INSERT
  ON "users"
  FOR EACH ROW
  EXECUTE PROCEDURE user_settings_insert_trigger_fnc();

  -- New Application Trigger via PgNotify

CREATE or REPLACE FUNCTION new_app_notification_trigger() RETURNS trigger AS $$
DECLARE
  id int;
  key varchar;
  value varchar;
BEGIN
--   IF TG_OP = 'INSERT' OR TG_OP = 'UPDATE' THEN
    id = NEW.application_id;
    key = NEW.application_slug;
    value = NEW.first_name;
--   ELSE
--     id = OLD.id;
--     key = OLD.key;
--     value = OLD.val;
--   END IF;
    PERFORM pg_notify('new_app_notification', json_build_object('table', TG_TABLE_NAME, 'id', id, 'application_slug', key, 'first_name', value, 'action_type', TG_OP)::text );
    RETURN NEW;
  END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER new_app_notification 
    AFTER INSERT 
    ON "applications"
    FOR EACH ROW 
    EXECUTE PROCEDURE new_app_notification_trigger();