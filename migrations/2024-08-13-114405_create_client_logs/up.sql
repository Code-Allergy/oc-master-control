CREATE TABLE client_logs (
    id BIGSERIAL PRIMARY KEY,
    client_id BIGINT NOT NULL,
    log_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    log_message TEXT NOT NULL,
    CONSTRAINT unique_log UNIQUE (client_id, log_time),
    CONSTRAINT fk_client FOREIGN KEY (client_id) REFERENCES clients (id) ON DELETE CASCADE
);

CREATE INDEX idx_client_id_log_time ON client_logs (client_id, log_time);
