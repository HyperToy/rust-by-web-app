create table tasks (
    id serial primary key,
    text text not null,
    completed boolean not null default false
)