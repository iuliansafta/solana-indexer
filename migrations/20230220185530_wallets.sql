-- Add migration script here
CREATE TABLE wallets (
	id uuid NOT NULL DEFAULT uuid_generate_v4(),
	address VARCHAR (50) UNIQUE NOT NULL,
	chain_name VARCHAR (255),
	created_on TIMESTAMP NOT NULL DEFAULT now(),
	fetched_on TIMESTAMP DEFAULT NULL,
	PRIMARY KEY(id)
);

CREATE OR REPLACE FUNCTION notify_wallet_changes()
RETURNS trigger AS $$
BEGIN
  PERFORM pg_notify(
    'wallet_changed',
    json_build_object(
      'operation', TG_OP,
      'record', row_to_json(NEW)
    )::text
  );

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER wallets_changed
AFTER INSERT OR UPDATE
ON wallets
FOR EACH ROW
EXECUTE PROCEDURE notify_wallet_changes();