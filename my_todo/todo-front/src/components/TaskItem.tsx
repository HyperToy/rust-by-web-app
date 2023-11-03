import { ChangeEventHandler, FC } from "react";
import { Task } from "../types/task";
import { Button, Card, Checkbox, Grid, Stack, Typography } from "@mui/material";

type Props = {
    task: Task,
    onUpdate: (task: Task) => void,
    onDelete: (id: number) => void,
};

const TaskItem: FC<Props> = ({ task, onUpdate, onDelete }) => {
    const handleCompletedCheckbox: ChangeEventHandler = (e) => {
        onUpdate({
            ...task,
            completed: !task.completed
        });
    };

    const handleDelete = () => onDelete(task.id);

    return (
        <Card sx={{ p: 1 }}>
            <Grid container spacing={2} alignItems="center">
                <Grid item xs={1}>
                    <Checkbox
                        onChange={handleCompletedCheckbox}
                        checked={task.completed}
                    />
                </Grid>
                <Grid item xs={9}>
                    <Stack spacing={1}>
                        <Typography variant="caption" fontSize={16}>
                            {task.text}
                        </Typography>
                    </Stack>
                </Grid>
                <Grid item xs={1}>
                    <Button onClick={handleDelete} color="error">
                        delete
                    </Button>
                </Grid>
            </Grid>
        </Card>
    )
};

export default TaskItem;
