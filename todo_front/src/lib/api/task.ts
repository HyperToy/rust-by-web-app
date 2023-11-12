import { NewTaskPayload, Task, UpdateTaskPayload } from "../../types/task";

export const addTaskItem = async (payload: NewTaskPayload) => {
    const res = await fetch('http://localhost:3000/task', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(payload),
    });
    if (!res.ok) {
        throw new Error('add task request failed');
    }
    const json: Task = await res.json();
    return json;
};

export const getTaskItems = async () => {
    const res = await fetch('http://localhost:3000/task');
    if (!res.ok) {
        throw new Error('get task request failed');
    }
    const json: Task[] = await res.json();
    return json;
};

export const updateTaskItem = async (task: UpdateTaskPayload) => {
    const { id, ...updateTask } = task;
    const res = await fetch(`http://localhost:3000/task/${id}`, {
        method: 'PATCH',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(updateTask),
    });
    if (!res) {
        throw new Error('update task request failed');
    }
    const json: Task = await res.json();
    return json;
};

export const deleteTaskItem = async (id: number) => {
    const res = await fetch(`http://localhost:3000/task/${id}`, {
        method: 'DELETE',
    });
    if (!res.ok) {
        throw new Error('delete task request failed');
    }
};
