-- Generated from pg_dump
-- We made a lot of modifications to the database during development

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

SET default_tablespace = '';
SET default_table_access_method = heap;

-- articles

CREATE TABLE public.articles (
                                 id integer NOT NULL,
                                 title text NOT NULL,
                                 links text[],
                                 condensed_links integer[] NOT NULL
);

ALTER TABLE public.articles OWNER TO postgres;

-- articles id sequence

CREATE SEQUENCE public.articles_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER TABLE public.articles_id_seq OWNER TO postgres;
ALTER SEQUENCE public.articles_id_seq OWNED BY public.articles.id;

-- requests

CREATE TABLE public.requests (
                                 pl_continue text,
                                 gap_continue text,
                                 continue text,
                                 created_at integer not null
);

ALTER TABLE public.requests OWNER TO postgres;

-- requests "timestamp" sequence (actually serial id)

CREATE SEQUENCE public.requests_timestamp_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER TABLE public.requests_timestamp_seq OWNER TO postgres;
ALTER SEQUENCE public.requests_timestamp_seq OWNED BY public.requests.created_at;

ALTER TABLE ONLY public.articles ALTER COLUMN id SET DEFAULT nextval('public.articles_id_seq'::regclass);
ALTER TABLE ONLY public.requests ALTER COLUMN created_at SET DEFAULT nextval('public.requests_timestamp_seq'::regclass);

ALTER TABLE ONLY public.articles
    ADD CONSTRAINT articles_pkey PRIMARY KEY (title);

-- index article titles to *maybe* speed up some operations
CREATE INDEX articles_title_idx ON public.articles USING gin (tite);
