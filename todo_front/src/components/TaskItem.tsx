import { ChangeEventHandler, FC, useEffect, useState } from "react";
import { Task } from "../types/task";
import { Box, Button, Card, Checkbox, Grid, Modal, Stack, TextField, Typography } from "@mui/material";
import { modalInnerStyle } from "../styles/modal";

type Props = {
    task: Task;
    onUpdate: (task: Task) => void;
    onDelete: (id: number) => void;
};

const TaskItem: FC<Props> = ({ task, onUpdate, onDelete }) => {
    const [editing, setEditing] = useState(false);
    const [editText, setEditText] = useState('');
    useEffect(() => {
        setEditText(task.text);
    }, [task]);

    const handleCompletedCheckbox: ChangeEventHandler = (_e) => {
        onUpdate({
            ...task,
            completed: !task.completed,
        });
    };

    const onCloseEditModal = () => {
        onUpdate({
            ...task,
            text: editText,
        });
        setEditing(false);
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
                <Grid item xs={8}>
                    <Stack spacing={1}>
                        <Typography variant="caption" fontSize={16}>
                            {task.text}
                        </Typography>
                    </Stack>
                </Grid>
                <Grid item xs={2}>
                    <Stack direction="row" spacing={1}>
                        <Button onClick={() => setEditing(true)} color="info">
                            edit
                        </Button>
                        <Button onClick={handleDelete} color="error">
                            delete
                        </Button>
                    </Stack>
                </Grid>
            </Grid>
            <Modal open={editing} onClose={onCloseEditModal}>
                <Box sx={modalInnerStyle}>
                    <Stack spacing={2}>
                        <TextField
                            size="small"
                            label="task text"
                            defaultValue={task.text}
                            onChange={(e) => setEditText(e.target.value)}
                        />
                    </Stack>
                </Box>
            </Modal>
        </Card>
    );
};

export default TaskItem;
