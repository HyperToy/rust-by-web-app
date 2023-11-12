import { FC } from "react";
import { Task } from "../types/task";
import { Stack, Typography } from "@mui/material";
import TaskItem from "./TaskItem";

type Props = {
    tasks: Task[];
    onUpdate: (task: Task) => void;
    onDelete: (id: number) => void;
};

const TaskList: FC<Props> = ({ tasks, onUpdate, onDelete }) => {
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
                    />
                ))}
            </Stack>
        </Stack>
    );
};

export default TaskList;