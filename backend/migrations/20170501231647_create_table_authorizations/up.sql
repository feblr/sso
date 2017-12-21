-- Table: sso.authorizations

CREATE TABLE IF NOT EXISTS sso.authorizations
(
  id bigserial NOT NULL,
  user_id bigint NOT NULL,
  client_id bigint NOT NULL,
  scope_id bigint NOT NULL,
  created_time timestamp with time zone NOT NULL DEFAULT now(),
  updated_time timestamp with time zone,
  removed_time timestamp with time zone,
  status integer NOT NULL DEFAULT 0,
  CONSTRAINT authorizations_pkey PRIMARY KEY (id),
  CONSTRAINT authorizations_unique_key UNIQUE (user_id, client_id, scope_id),
  CONSTRAINT authorizations_user_id_fkey FOREIGN KEY (user_id) REFERENCES sso.users (id) ON UPDATE NO ACTION ON DELETE NO ACTION,
  CONSTRAINT authorizations_client_id_fkey FOREIGN KEY (client_id) REFERENCES sso.applications (id) ON UPDATE NO ACTION ON DELETE NO ACTION,
  CONSTRAINT authorizations_scope_id_fkey FOREIGN KEY (scope_id) REFERENCES sso.scopes (id) ON UPDATE NO ACTION ON DELETE NO ACTION
)
WITH (
  OIDS=FALSE
);