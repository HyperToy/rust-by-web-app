import { Label, NewLabelPayload } from "../../types/task";

export const getLabelItems = async () => {
    const res = await fetch('http://localhost:3000/label');
    if (!res.ok) {
        throw new Error('get label request failed');
    }
    const json: Label[] = await res.json();
    return json;
};

export const addLabelItem = async (payload: NewLabelPayload) => {
    const res = await fetch('http://localhost:3000/label', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(payload),
    });
    if (!res.ok) {
        throw new Error('add label request failed');
    }
    const json: Label = await res.json();
    return json;
};

export const deleteLabelItem = async (id: number) => {
    const res = await fetch(`http://localhost:3000/label/${id}`, {
        method: 'DELETE',
    });
    if (!res.ok) {
        throw new Error('delete label request failed');
    }
};
