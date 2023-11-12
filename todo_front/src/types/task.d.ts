export type Task = {
    id: number;
    text: string;
    completed: boolean;
    labels: Label[];
};

export type NewTaskPayload = {
    text: string;
};

export type Label = {
    id: number;
    name: string;
};

export type NewLabelPayload = {
    name: string;
};
