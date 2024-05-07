-- Create and populate database
CREATE TABLE IF NOT EXISTS links (
    uuid VARCHAR(36) PRIMARY KEY NOT NULL,
    short VARCHAR(32) NOT NULL,
    target VARCHAR(32768) NOT NULL
);

CREATE UNIQUE INDEX short_idx on links(short);

INSERT INTO links
    (uuid, short, target)
VALUES (
    "018f244b-942b-7007-927b-ace4fadf4a88",
    "6fy",
    "https://mailman.bitfolk.com/mailman/hyperkitty/list/users@mailman.bitfolk.com/message/BV6BHVJN7YL4OYN7C5Y5LRPWJKALPWY6/"
);
