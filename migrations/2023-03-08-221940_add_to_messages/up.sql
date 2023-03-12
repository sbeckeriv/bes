-- Your SQL goes here
ALTER TABLE messages 
ADD COLUMN message_to text;

ALTER TABLE messages 
ADD COLUMN message_cc text;

ALTER TABLE messages 
ADD COLUMN message_bcc text;

ALTER TABLE messages 
ADD COLUMN parent_thread_key text;