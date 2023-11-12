import { FC } from "react";
import { Stack, Typography } from "@mui/material";
import { Label, Task, UpdateTaskPayload } from "../types/task";
import TaskItem from "./TaskItem";

type Props = {
    tasks: Task[];
    labels: Label[];
    onUpdate: (task: UpdateTaskPayload) => void;
    onDelete: (id: number) => void;
};

const TaskList: FC<Props> = ({ tasks, labels, onUpdate, onDelete }) => {
    return (
        <Stack spacing={2}>
            <Typography variant="h2">task list</Typography>
            <Stack spacing={2}>
                {tasks.map((task) => (
                    <TaskItem
                        key={task.id}
                        task={task}
                        onUpdate={onUpdate}
                        onDelete={onDelete}
                        labels={labels}
                    />
                ))}
            </Stack>
        </Stack>
    );
};

export default TaskList;