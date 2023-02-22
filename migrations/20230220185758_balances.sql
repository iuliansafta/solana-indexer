-- Add migration script here
CREATE TABLE balances (
	id uuid NOT NULL DEFAULT uuid_generate_v4(),
	address_id uuid,
	pre_balance BIGINT,
	post_balance BIGINT,
	fee BIGINT,
	transaction_hash TEXT,
	transfer_type SMALLINT,
	block_time TIMESTAMP NOT NULL,
	created_on TIMESTAMP NOT NULL default now(),
    metadata JSONB DEFAULT NULL,
	PRIMARY key (id),
	CONSTRAINT fk_addresses 
		FOREIGN KEY(address_id) 
		REFERENCES wallets(id)
)