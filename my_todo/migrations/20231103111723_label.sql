create table labels (
    id serial primary key,
    name text not null
);

create table task_labels (
    id serial primary key,
    task_id integer not null references tasks (id) deferrable initially deferred,
    label_id integer not null references labels (id) deferrable initially deferred
);