import { FC, useEffect, useState } from 'react';
import 'modern-css-reset';
import { ThemeProvider, createTheme } from '@mui/material/styles';
import { Box, Stack, Typography } from '@mui/material';
import { Label, NewLabelPayload, NewTaskPayload, Task } from './types/task';
import TaskList from './components/TaskList.tsx';
import TaskForm from './components/TaskForm.tsx';
import { addTaskItem, deleteTaskItem, getTaskItems, updateTaskItem } from './lib/api/task.ts';
import SideNav from './components/SideNav.tsx';
import { addLabelItem, deleteLabelItem, getLabelItems } from './lib/api/label.ts';

const TodoApp: FC = () => {
    const [tasks, setTasks] = useState<Task[]>([]);
    const [labels, setLabels] = useState<Label[]>([]);
    const [filterLabelId, setFilterLabelId] = useState<number | null>(null);

    const onSubmit = async (payload: NewTaskPayload) => {
        if (!payload.text) {
            return;
        }

        await addTaskItem(payload);

        const tasks = await getTaskItems();
        setTasks(tasks);
    };

    const onUpdate = async (updateTask: Task) => {
        await updateTaskItem(updateTask);

        const tasks = await getTaskItems();
        setTasks(tasks);
    };

    const onDelete = async (id: number) => {
        await deleteTaskItem(id);

        const tasks = await getTaskItems();
        setTasks(tasks);
    };

    // filtering tasks by selected label
    const onSelectLabel = (label: Label | null) => {
        setFilterLabelId(label?.id ?? null);
    };

    const onSubmitNewLabel = async (newLabel: NewLabelPayload) => {
        if (!labels.some((label) => label.name === onSubmitNewLabel.name)) {
            const res = await addLabelItem(newLabel);
            setLabels([...labels, res]);
        }
    };

    const onDeleteLabel = async (id: number) => {
        await deleteLabelItem(id);
        setLabels((prev) => prev.filter((label) => label.id !== id));

    };

    const tasksToDisplay = filterLabelId
        ? tasks.filter((task) => task.labels.some((label) => label.id === filterLabelId))
        : tasks;

    useEffect(() => {
        ; (async () => {
            const tasks = await getTaskItems();
            setTasks(tasks);
            const labelResponse = await getLabelItems();
            setLabels(labelResponse);
        })();
    }, []);

    return (
        <>
            <Box
                sx={{
                    backgroundColor: 'white',
                    borderBottom: '1px solid gray',
                    display: 'flex',
                    alignItems: 'center',
                    position: 'fixed',
                    top: 0,
                    p: 2,
                    width: '100%',
                    height: 80,
                    zIndex: 3,
                }}
            >
                <Typography variant="h1">Todo App</Typography>
            </Box>
            <Box
                sx={{
                    backgroundColor: 'white',
                    borderRight: '1px solid gray',
                    position: 'fixed',
                    height: 'calc(100%-80px)',
                    width: 200,
                    zIndex: 2,
                    left: 0,
                }}
            >
                <SideNav
                    labels={labels}
                    onSelectLabel={onSelectLabel}
                    filterLabelId={filterLabelId}
                    onSubmitNewLabel={onSubmitNewLabel}
                    onDeleteLabel={onDeleteLabel}
                />
            </Box>
            <Box
                sx={{
                    display: 'flex',
                    justifyContent: 'center',
                    p: 5,
                    mt: 10,
                }}
            >
                <Box maxWidth={700} width="100%">
                    <Stack spacing={5}>
                        <TaskForm onSubmit={onSubmit} />
                        <TaskList tasks={tasksToDisplay} onUpdate={onUpdate} onDelete={onDelete} />
                    </Stack>
                </Box>
            </Box>
        </>
    );
};

const theme = createTheme({
    typography: {
        h1: {
            fontSize: 30,
        },
        h2: {
            fontSize: 20,
        },
    },
});

const App: FC = () => {
    return (
        <ThemeProvider theme={theme}>
            <TodoApp />
        </ThemeProvider>
    );
};

export default App;
