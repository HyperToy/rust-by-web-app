import { ChangeEventHandler, FC, useEffect, useState } from "react";
import { Box, Button, Card, Checkbox, Chip, FormControlLabel, Grid, Modal, Stack, TextField, Typography } from "@mui/material";
import { modalInnerStyle } from "../styles/modal";
import { Label, Task, UpdateTaskPayload } from "../types/task";
import { toggleLabels } from "../lib/toggleLabels";

type Props = {
    task: Task;
    onUpdate: (task: UpdateTaskPayload) => void;
    onDelete: (id: number) => void;
    labels: Label[];
};

const TaskItem: FC<Props> = ({ task, onUpdate, onDelete, labels }) => {
    const [editing, setEditing] = useState(false);
    const [editText, setEditText] = useState('');
    const [editLabels, setEditLabels] = useState<Label[]>([]);

    useEffect(() => {
        setEditText(task.text);
        setEditLabels(task.labels);
    }, [task, editing]);

    const handleCompletedCheckbox: ChangeEventHandler = (_e) => {
        onUpdate({
            ...task,
            completed: !task.completed,
            labels: task.labels.map((label) => label.id),
        });
    };

    const onCloseEditModal = () => {
        console.log('ok to here');
        onUpdate({
            id: task.id,
            text: editText,
            completed: task.completed,
            labels: editLabels.map((label) => label.id),
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
                        <Stack direction="row" spacing={1}>
                            {task.labels?.map((label) => (
                                <Chip key={label.id} label={label.name} size="small" />
                            ))}
                        </Stack>
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
                        <Stack>
                            <Typography variant="subtitle1">Labels</Typography>
                            {labels.map((label) => (
                                <FormControlLabel
                                    key={label.id}
                                    control={
                                        <Checkbox
                                            defaultChecked={task.labels.some((taskLabel) => taskLabel.id === label.id)}
                                        />
                                    }
                                    label={label.name}
                                    onChange={() => setEditLabels((prev) => toggleLabels(prev, label))}
                                />
                            ))}
                        </Stack>
                    </Stack>
                </Box>
            </Modal>
        </Card>
    );
};

export default TaskItem;
