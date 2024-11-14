CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS Users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name text NOT NULL,
    username text UNIQUE NOT NULL,
    email text NOT NULL UNIQUE,
    email_hash text not null unique,
    password text NOT NULL,
    bio text NULL,
    image text NULL,
    created_at TIMESTAMPTZ NOT NULL default NOW(),
    updated_at TIMESTAMPTZ NOT NULL default NOW()
);

create unique index idx_users_email on users(email);
create unique index idx_users_username on users(username);

CREATE TABLE IF NOT EXISTS Follows (
    follow_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    follower_id UUID NOT NULL REFERENCES Users(id) ON DELETE CASCADE ON UPDATE CASCADE,
    influencer_id UUID NOT NULL REFERENCES Users(id) ON DELETE CASCADE ON UPDATE CASCADE,
    created_at TIMESTAMPTZ NOT NULL default NOW(),
    updated_at TIMESTAMPTZ NOT NULL default NOW()
);

create unique index follows_follower_influencer on follows(follower_id, influencer_id);

CREATE TABLE IF NOT EXISTS Articles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    slug text UNIQUE NOT NULL,
    author_id UUID NOT NULL REFERENCES Users(id) ON DELETE CASCADE ON UPDATE CASCADE,
    title text NOT NULL,
    description text NOT NULL,
    body text NOT NULL,
    tags text[],
    cover_image text,
    reading_time int,
    created_at TIMESTAMPTZ NOT NULL default NOW(),
    updated_at TIMESTAMPTZ NOT NULL default NOW()
);

create unique index idx_articles_slug on articles(slug);
create index idx_articles_tags on articles  using gin(tags);

CREATE TABLE IF NOT EXISTS ArticleTags (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    article_id UUID NOT NULL REFERENCES Articles(id) ON DELETE CASCADE ON UPDATE CASCADE,
    tag text NOT NULL
);

CREATE INDEX IF NOT EXISTS tags ON ArticleTags (tag);

CREATE TABLE IF NOT EXISTS FavArticles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    article_id UUID NOT NULL REFERENCES Articles(id) ON DELETE CASCADE ON UPDATE CASCADE,
    user_id UUID NOT NULL REFERENCES Users(id) ON DELETE CASCADE ON UPDATE CASCADE,
    created_at TIMESTAMPTZ NOT NULL default NOW(),
    updated_at TIMESTAMPTZ NOT NULL default NOW()
);

create unique index idx_favarticles_article_user on FavArticles(user_id, article_id);

CREATE TABLE IF NOT EXISTS Comments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    article_id UUID NOT NULL REFERENCES Articles(id) ON DELETE CASCADE ON UPDATE CASCADE,
    user_id UUID NOT NULL REFERENCES Users(id) ON DELETE CASCADE ON UPDATE CASCADE,
    body text NOT NULL,
    created_at TIMESTAMPTZ NOT NULL default NOW(),
    updated_at TIMESTAMPTZ NOT NULL default NOW()
);
