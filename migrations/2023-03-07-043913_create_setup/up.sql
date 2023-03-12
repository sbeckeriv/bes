-- Your SQL goes here
create table IF NOT EXISTS raw_messages( 
    	id INTEGER PRIMARY KEY AUTOINCREMENT,
        message_id text,
        message blob
);

create table if not EXISTS messages(
    	id INTEGER PRIMARY KEY AUTOINCREMENT,
        message_id text not null,
        account text not null,
        parent_id text ,
        subject text,
        sent_at text,
        message_from text,
        pinned_at text,
        done_at text,
        reminder_at text,
        folders text,
        content text,
        text_format text,
        html_format text
);