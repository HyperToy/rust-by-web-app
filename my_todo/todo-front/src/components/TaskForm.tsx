import { Box, Button, Grid, Paper, TextField } from "@mui/material";
import { FC, useState } from "react";
import { NewTaskPayload } from "../types/task";

type Props = {
    onSubmit: (newTask: NewTaskPayload) => void;
};

const TaskForm: FC<Props> = ({ onSubmit }) => {
    const [editText, setEditText] = useState('');

    const addTaskHandler = async () => {
        if (!editText) {
            return;
        }
        onSubmit({
            text: editText,
        });
        setEditText('');
    };

    return (
        <Paper elevation={2}>
            <Box sx={{ p: 2 }}>
                <Grid item xs={12}>
                    <TextField
                        label="new task text"
                        variant="filled"
                        value={editText}
                        onChange={(e) => setEditText(e.target.value)}
                        fullWidth
                    />
                </Grid>
                <Grid item xs={9} />
                <Grid item xs={3}>
                    <Button onClick={addTaskHandler} fullWidth>
                        add task
                    </Button>
                </Grid>
            </Box>
        </Paper>
    );
};

export default TaskForm;