<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";

    type Selector = {
        id: number,
        name: string,
    };

    type AnnualDate = {
        month: number,
        day: number,
    }

    type AnnualWindow = {
        start_date: AnnualDate,
        end_date: AnnualDate,
    }

    type Competition = {
        id: number,
        comp_name: string,
        season_window: AnnualWindow,
        min_no_of_teams: number,
        rank_criteria: string[],
        comp_type: any,
        parent_id: number,

        // GameRules
        periods: number,
        period_length: number,
        overtime_length: number,
        continuous_overtime: boolean,
    };

    const getCompSelector = (): Promise<Selector[]> => {
        return invoke("comp_select_package");
    };

    const createNewComp = async (event: Event): Promise<Competition> => {
        event.preventDefault();
        const comp: Competition = await invoke("comp_editor_package", { id: 0 });
        return comp;
    };
</script>

<main class="container">
    Competitions
    <select>
        {#await getCompSelector()}
            <option disabled>Loading...</option>
        {:then list}
            {#each list as comp}
                <option value={comp.id}>{comp.name}</option>
            {/each}
        {:catch error}
            <option disabled>Error loading competitions</option>
        {/await}
    </select>
    <button onclick={createNewComp}>New Competition</button>

    <!-- The editing fields. -->
     <form>
        <label for="comp-name">Name</label>
        <input type="text" id="comp-name" name="comp-name">
        <fieldset>
            <legend>Begin Date</legend>
            <label for="begin-month">Month</label>
            <select id="begin-month" name="begin-month">
                <option>Jan</option>
                <option>Feb</option>
                <option>Mar</option>
                <option>Apr</option>
                <option>May</option>
                <option>Jun</option>
                <option>Jul</option>
                <option>Aug</option>
                <option>Sep</option>
                <option>Oct</option>
                <option>Nov</option>
                <option>Dec</option>
            </select>
            <label for="begin-day">Day</label>
            <input type="number" min="1" max="31" id="begin-day" name="begin-day">
        </fieldset>
        <fieldset>
            <legend>End Date</legend>
            <label for="end-month">Month</label>
            <select id="end-month" name="end-month">
                <option>Jan</option>
                <option>Feb</option>
                <option>Mar</option>
                <option>Apr</option>
                <option>May</option>
                <option>Jun</option>
                <option>Jul</option>
                <option>Aug</option>
                <option>Sep</option>
                <option>Oct</option>
                <option>Nov</option>
                <option>Dec</option>
            </select>
            <label for="end-day">Day</label>
            <input type="number" min="1" max="31" id="end-day" name="end-day">
        </fieldset>
        <label for="min-no-of_teams">Min No. of Teams</label>
        <input type="number" id="min-no-of-teams" name="min-no-of-teams">
        <label for="rank-criteria">Rank Criteria</label>
        <input type="text" id="rank-criteria" name="rank-criteria"> <!-- Placeholder -->
        <label for="comp-type">Competition Type</label>
        <select id="comp-type" name="comp-type">
            <option>Container</option>
            <option>Tournament</option>
            <option>Round Robin</option>
            <option>Knockout Round</option>
        </select>
        <label for="parent-id">Parent ID</label>
        <input type="number" id="parent-id" name="parent-id">

        <fieldset>
            <legend>Game Rules</legend>
            <label for="periods">Periods</label>
            <input type="number" id="periods" name="periods">
            <label for="period-length">Period Length</label>
            <input type="number" id="period-length" name="period-length">
            <label for="overtime-length">Overtime Length</label>
            <input type="number" id="overtime-length" name="overtime-length">
            <input type="checkbox" id="continuous-overtime" name="continuous-overtime">
            <label for="continuous-overtime">Continuous Overtime</label>
        </fieldset>

        <fieldset>
            <legend>Round Robin Format</legend>
            <label for="rounds">Rounds</label>
            <input type="number" id="rounds" name="rounds">
            <label for="extra-matches">Extra Matches</label>
            <input type="number" id="extra-matches" name="extra-matches">
            <label for="points-for-win">Points for Win</label>
            <input type="number" id="points-for-win" name="points-for-win">
            <label for="points-for-ot-win">Points for Overtime Win</label>
            <input type="number" id="points-for-ot-win" name="points-for-ot-win">
            <label for="points-for-draw">Points for Draw</label>
            <input type="number" id="points-for-draw" name="points-for-draw">
            <label for="points-for-ot-loss">Points for OT Loss</label>
            <input type="number" id="points-for-ot-loss" name="points-for-ot-loss">
            <label for="points-for-loss">Points for Loss</label>
            <input type="number" id="points-for-loss" name="points-for-loss">
        </fieldset>

        <fieldset>
            <legend>Knockout Round Format</legend>
            <label for="wins-required">Wins Required</label>
            <input type="number" id="wins-required" name="wins-required">
        </fieldset>
        <input type="submit" value="Submit">
     </form>
</main>