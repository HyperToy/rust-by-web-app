import { FC, useEffect, useState } from 'react'
import 'modern-css-reset'
import { ThemeProvider, createTheme } from '@mui/material/styles';
import { Box, Stack, Typography } from '@mui/material';
import { NewTaskPayload, Task } from './types/task';
import TaskList from './components/TaskList.tsx';
import TaskForm from './components/TaskForm.tsx';
import { addTaskItem, getTaskItems, updateTaskItem } from './lib/api/task.ts';

const TodoApp: FC = () => {
    const [tasks, setTasks] = useState<Task[]>([]);

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

    useEffect(() => {
        ; (async () => {
            const tasks = await getTaskItems();
            setTasks(tasks);
        })();
    }, []);

    return (
        <>
            <Box
                sx={{
                    backgroundColor: 'white',
                    borderBottom: '1px solid gray',
                    display: 'flex',
                    alignItem: 'center',
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
                    display: 'flex',
                    justifyContent: 'center',
                    p: 5,
                    mt: 10,
                }}
            >
                <Box maxWidth={700} width="100%">
                    <Stack spacing={5}>
                        <TaskForm onSubmit={onSubmit} />
                        <TaskList tasks={tasks} onUpdate={onUpdate} />
                    </Stack>
                </Box>
            </Box>
        </>
    )
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
}

export default App
