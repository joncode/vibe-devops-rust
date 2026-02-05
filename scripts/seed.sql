-- Seed script for vibe-devops-server
-- Creates test users, identifiers, passwords, session tokens, and refresh tokens

-- Enable pgcrypto for gen_random_bytes
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Clean up existing seed data (optional - comment out if you want to keep existing data)
-- DELETE FROM app_refresh_tokens;
-- DELETE FROM app_session_tokens;
-- DELETE FROM app_user_passwords;
-- DELETE FROM app_social_identifiers;
-- DELETE FROM app_users WHERE display_name LIKE 'Test User%';

-- ============================================================================
-- Create Test Users
-- ============================================================================

DO $$
DECLARE
    user_ids UUID[] := ARRAY[]::UUID[];
    new_user_id UUID;
    i INT;
BEGIN
    -- Create 100 test users
    FOR i IN 1..100 LOOP
        INSERT INTO app_users (display_name, avatar_url, metadata, status)
        VALUES (
            'Test User ' || i,
            'https://api.dicebear.com/7.x/avataaars/svg?seed=' || i,
            jsonb_build_object(
                'test', true,
                'seed_number', i,
                'created_by', 'seed_script'
            ),
            CASE 
                WHEN i % 10 = 0 THEN 'suspended'::user_status
                WHEN i % 5 = 0 THEN 'inactive'::user_status
                WHEN i % 3 = 0 THEN 'pending'::user_status
                ELSE 'active'::user_status
            END
        )
        RETURNING id INTO new_user_id;
        
        user_ids := array_append(user_ids, new_user_id);
        
        -- Create email identifier for each user
        INSERT INTO app_social_identifiers (user_id, identifier_type, identifier_value, verified, is_primary, metadata)
        VALUES (
            new_user_id,
            'email'::identifier_type,
            'testuser' || i || '@example.com',
            i % 2 = 0,  -- Every other user is verified
            TRUE,
            jsonb_build_object('source', 'seed_script')
        );
        
        -- Create password for each user (static hash representing 'password123')
        INSERT INTO app_user_passwords (user_id, password_hash)
        VALUES (
            new_user_id,
            '$argon2id$v=19$m=19456,t=2,p=1$c2VlZF9zYWx0XzEyMzQ1Ng$dGVzdF9oYXNoX2Zvcl9zZWVkX3VzZXJfcGFzc3dvcmQ'
        );
        
        -- Some users get additional identifiers
        IF i % 3 = 0 THEN
            INSERT INTO app_social_identifiers (user_id, identifier_type, identifier_value, verified, is_primary, metadata)
            VALUES (
                new_user_id,
                'phone'::identifier_type,
                '+1555' || LPAD(i::TEXT, 7, '0'),
                i % 4 = 0,
                FALSE,
                jsonb_build_object('source', 'seed_script')
            );
        END IF;
        
        IF i % 5 = 0 THEN
            INSERT INTO app_social_identifiers (user_id, identifier_type, identifier_value, verified, is_primary, metadata)
            VALUES (
                new_user_id,
                'oauth_github'::identifier_type,
                'github_user_' || i,
                TRUE,
                FALSE,
                jsonb_build_object('github_id', 1000000 + i)
            );
        END IF;
    END LOOP;

    RAISE NOTICE 'Created % test users', array_length(user_ids, 1);
    
    -- ========================================================================
    -- Generate 10,000 Session Tokens
    -- ========================================================================
    
    RAISE NOTICE 'Generating 10,000 session tokens...';
    
    INSERT INTO app_session_tokens (user_id, token_hash, device_info, ip_address, user_agent, expires_at, revoked_at, last_used_at, created_at)
    SELECT 
        user_ids[1 + (random() * 99)::INT],
        encode(sha256(gen_random_bytes(32)), 'hex'),
        jsonb_build_object(
            'device', CASE (random() * 4)::INT
                WHEN 0 THEN 'iPhone'
                WHEN 1 THEN 'Android'
                WHEN 2 THEN 'MacOS'
                WHEN 3 THEN 'Windows'
                ELSE 'Linux'
            END,
            'browser', CASE (random() * 3)::INT
                WHEN 0 THEN 'Chrome'
                WHEN 1 THEN 'Firefox'
                WHEN 2 THEN 'Safari'
                ELSE 'Edge'
            END,
            'app_version', '1.' || (random() * 10)::INT || '.' || (random() * 20)::INT
        ),
        (192 + (random() * 63)::INT)::TEXT || '.' || 
        (random() * 255)::INT::TEXT || '.' || 
        (random() * 255)::INT::TEXT || '.' || 
        (1 + random() * 254)::INT::TEXT,
        CASE (random() * 5)::INT
            WHEN 0 THEN 'Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15'
            WHEN 1 THEN 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/120.0.0.0'
            WHEN 2 THEN 'Mozilla/5.0 (Macintosh; Intel Mac OS X 14_0) AppleWebKit/605.1.15 Safari/605.1.15'
            WHEN 3 THEN 'Mozilla/5.0 (Linux; Android 14) AppleWebKit/537.36 Chrome/120.0.0.0 Mobile'
            ELSE 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/120.0.0.0'
        END,
        -- expires_at: between now and 30 days from now
        NOW() + (random() * 30 || ' days')::INTERVAL,
        -- revoked_at: ~20% are revoked
        CASE WHEN random() < 0.2 THEN NOW() - (random() * 10 || ' days')::INTERVAL ELSE NULL END,
        -- last_used_at: random time in the past week
        NOW() - (random() * 7 || ' days')::INTERVAL,
        -- created_at: random time in the past 60 days
        NOW() - (random() * 60 || ' days')::INTERVAL
    FROM generate_series(1, 10000);

    RAISE NOTICE 'Created 10,000 session tokens';

    -- ========================================================================
    -- Generate 10,000 Refresh Tokens
    -- ========================================================================
    
    RAISE NOTICE 'Generating 10,000 refresh tokens...';
    
    INSERT INTO app_refresh_tokens (user_id, token_hash, session_id, family_id, generation, ip_address, user_agent, expires_at, revoked_at, created_at)
    SELECT 
        user_ids[1 + (random() * 99)::INT],
        encode(sha256(gen_random_bytes(32)), 'hex'),
        NULL,  -- session_id: we'll leave most null for simplicity
        gen_random_uuid(),  -- family_id
        1 + (random() * 5)::INT,  -- generation 1-5
        (192 + (random() * 63)::INT)::TEXT || '.' || 
        (random() * 255)::INT::TEXT || '.' || 
        (random() * 255)::INT::TEXT || '.' || 
        (1 + random() * 254)::INT::TEXT,
        CASE (random() * 5)::INT
            WHEN 0 THEN 'Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15'
            WHEN 1 THEN 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/120.0.0.0'
            WHEN 2 THEN 'Mozilla/5.0 (Macintosh; Intel Mac OS X 14_0) AppleWebKit/605.1.15 Safari/605.1.15'
            WHEN 3 THEN 'Mozilla/5.0 (Linux; Android 14) AppleWebKit/537.36 Chrome/120.0.0.0 Mobile'
            ELSE 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/120.0.0.0'
        END,
        -- expires_at: between now and 90 days from now (refresh tokens last longer)
        NOW() + (random() * 90 || ' days')::INTERVAL,
        -- revoked_at: ~15% are revoked
        CASE WHEN random() < 0.15 THEN NOW() - (random() * 10 || ' days')::INTERVAL ELSE NULL END,
        -- created_at: random time in the past 90 days
        NOW() - (random() * 90 || ' days')::INTERVAL
    FROM generate_series(1, 10000);

    RAISE NOTICE 'Created 10,000 refresh tokens';
    RAISE NOTICE 'Seed complete!';
    
END $$;

-- Show final counts
SELECT 'app_users' AS table_name, COUNT(*) AS count FROM app_users
UNION ALL
SELECT 'app_social_identifiers', COUNT(*) FROM app_social_identifiers
UNION ALL
SELECT 'app_user_passwords', COUNT(*) FROM app_user_passwords
UNION ALL
SELECT 'app_session_tokens', COUNT(*) FROM app_session_tokens
UNION ALL
SELECT 'app_refresh_tokens', COUNT(*) FROM app_refresh_tokens;
