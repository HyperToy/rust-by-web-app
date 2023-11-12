import { Box, Button, Checkbox, Chip, FormControlLabel, Grid, Modal, Paper, Stack, TextField } from "@mui/material";
import { FC, useState } from "react";
import { Label, NewTaskPayload } from "../types/task";
import { modalInnerStyle } from "../styles/modal";
import { toggleLabels } from "../lib/toggleLabels";

type Props = {
    onSubmit: (newTask: NewTaskPayload) => void;
    labels: Label[];
};

const TaskForm: FC<Props> = ({ onSubmit, labels }) => {
    const [editText, setEditText] = useState('');
    const [editLabels, setEditLabels] = useState<Label[]>([]);
    const [openLabelModal, setOpenLabelModal] = useState(false);

    const addTaskHandler = async () => {
        if (!editText) {
            return;
        }
        onSubmit({
            text: editText,
            labels: editLabels.map((label) => label.id),
        });
        setEditText('');
    };

    return (
        <Paper elevation={2}>
            <Box sx={{ p: 2 }}>
                <Grid container rowSpacing={2} columnSpacing={5}>
                    <Grid item xs={12}>
                        <TextField
                            label="new task text"
                            variant="filled"
                            value={editText}
                            onChange={(e) => setEditText(e.target.value)}
                            fullWidth
                        />
                    </Grid>
                    <Grid item xs={12}>
                        <Stack direction="row" spacing={1}>
                            {editLabels.map((label) => (
                                <Chip key={label.id} label={label.name} />
                            ))}
                        </Stack>
                    </Grid>
                    <Grid item xs={3} xl={7}>
                        <Button
                            onClick={() => setOpenLabelModal(true)}
                            fullWidth
                            color="secondary"
                        >
                            select label
                        </Button>
                    </Grid>
                    <Grid item xs={6} />
                    <Grid item xs={3}>
                        <Button onClick={addTaskHandler} fullWidth>
                            add task
                        </Button>
                    </Grid>
                    <Modal open={openLabelModal} onClose={() => setOpenLabelModal(false)}>
                        <Box sx={modalInnerStyle}>
                            <Stack>
                                {labels.map((label) => (
                                    <FormControlLabel
                                        key={label.id}
                                        control={<Checkbox checked={editLabels.includes(label)} />}
                                        label={label.name}
                                        onChange={() =>
                                            setEditLabels((prev) => toggleLabels(prev, label))
                                        }
                                    />
                                ))}
                            </Stack>
                        </Box>
                    </Modal>
                </Grid>
            </Box>
        </Paper>
    );
};

export default TaskForm;