import { FC } from "react";
import { Task } from "../types/task"
import { Card, Checkbox, Stack, Typography } from "@mui/material";

type Props = {
    tasks: Task[],
    onUpdate: (task: Task) => void,
};

const TaskList: FC<Props> = ({ tasks, onUpdate }) => {

    const handleCompletedCheckbox = (task: Task) => {
        onUpdate({
            ...task,
            completed: !task.completed,
        })
    };

    return (
        <Stack spacing={2}>
            <Typography variant="h2">task list</Typography>
            <Stack spacing={2}>
                {tasks.map((task) => (
                    <Card key={task.id} sx={{ p: 2 }}>
                        <Stack direction="row" alignItems="center">
                            <Checkbox
                                checked={task.completed}
                                onChange={() => handleCompletedCheckbox(task)} />
                            <Typography variant="body1">{task.text}</Typography>
                        </Stack>
                    </Card>
                ))}
            </Stack>
        </Stack>
    );
};

export default TaskList;