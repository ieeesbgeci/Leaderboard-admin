CREATE TABLE events (
    id UUID PRIMARY KEY,
    name TEXT,
    logo TEXT,
    event_type TEXT
);

CREATE TABLE teams (
    id UUID PRIMARY KEY,
    name TEXT,
    score INTEGER,
    logo TEXT
);

CREATE TABLE users (
    id UUID PRIMARY KEY,
    name TEXT,
    score INTEGER,
    logo TEXT
);

CREATE TABLE event_teams (
    event_id UUID,
    team_id UUID,
    PRIMARY KEY (event_id, team_id),
    FOREIGN KEY (event_id) REFERENCES events (id),
    FOREIGN KEY (team_id) REFERENCES teams (id)
);

CREATE TABLE team_members (
    team_id UUID,
    user_id UUID,
    PRIMARY KEY (team_id, user_id),
    FOREIGN KEY (team_id) REFERENCES teams (id),
    FOREIGN KEY (user_id) REFERENCES users (id),
    UNIQUE (user_id)
);

CREATE TABLE event_users (
    user_id UUID,
    event_id UUID,
    PRIMARY KEY (user_id, event_id),
    FOREIGN KEY (user_id) REFERENCES users (id),
    FOREIGN KEY (event_id) REFERENCES events (id)
);

