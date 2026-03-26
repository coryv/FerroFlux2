export interface UndoableAction {
    execute(): Promise<void>;
    undo(): Promise<void>;
    description: string;
}

export class UndoManager {
    past = $state<UndoableAction[]>([]);
    future = $state<UndoableAction[]>([]);
    maxHistory = 50;

    get canUndo() { return this.past.length > 0; }
    get canRedo() { return this.future.length > 0; }

    async perform(action: UndoableAction) {
        await action.execute();
        
        let newPast = [...this.past, action];
        if (newPast.length > this.maxHistory) {
            newPast = newPast.slice(newPast.length - this.maxHistory);
        }
        
        this.past = newPast;
        this.future = [];
    }

    async undo() {
        if (!this.canUndo) return;
        const action = this.past.pop();
        if (action) {
            await action.undo();
            this.future.push(action);
            
            // Svelte 5 array reactivity
            this.past = [...this.past];
            this.future = [...this.future];
        }
    }

    async redo() {
        if (!this.canRedo) return;
        const action = this.future.pop();
        if (action) {
            await action.execute();
            this.past.push(action);
            
            // Svelte 5 array reactivity
            this.past = [...this.past];
            this.future = [...this.future];
        }
    }
    
    clear() {
        this.past = [];
        this.future = [];
    }
}
