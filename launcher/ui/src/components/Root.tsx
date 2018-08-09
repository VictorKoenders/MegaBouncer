import * as React from "react";

export interface Props { }
export interface State {
    state: server.State | null,
    open_uuids: string[],
}

export class Root extends React.Component<Props, State> {
    interval: number;
    constructor(props: Props, context?: any) {
        super(props, context);
        this.state = {
            state: null,
            open_uuids: [],
        };
    }
    componentWillMount() {
        this.interval = setInterval(this.fetch.bind(this), 1000);
        this.fetch();
    }
    fetch() {
        fetch("/api/state")
            .then(r => r.json())
            .then((r: server.State) => {
                this.setState({
                    state: r
                });
            });
    }
    render_time(diff: number) {
        diff = Math.ceil(diff / 1000);
        var result = "";
        if (diff > 3600) {
            let hours = Math.floor(diff / 3600);
            diff -= hours * 3600;
            result += hours + " hours";
        }
        if (diff > 60) {
            if (result) result += ", ";
            let minutes = Math.floor(diff / 60);
            diff -= minutes * 60;
            result += diff + " minutes";
        }
        if (diff > 0) {
            if (result) result += ", ";
            result += diff + " seconds";
        }
        return result;
    }
    render_running_build(build: server.RunningBuild, index: number): JSX.Element {
        let start = new Date(build.started_on);
        let diff = Date.now() - start.getTime();
        return <div key={index}>
            <p onClick={this.toggle_open.bind(this, build.uuid)}>
                <b>{build.build}</b> (running for {this.render_time(diff)})
            </p>
            <pre>{build.stdout}</pre>
            <pre>{build.stderr}</pre>
        </div>;
    }
    render_finished_build(build: server.FinishedBuild, index: number): JSX.Element {
        let start = new Date(build.started_on);
        let end = new Date(build.ended_on);
        let diff = end.getTime() - start.getTime();
        let is_open = this.state.open_uuids.some(u => u == build.uuid);
        if (is_open) {
            return <div key={index}>
                <p onClick={this.toggle_open.bind(this, build.uuid)}>
                    <b>{build.build}</b> (finished in {this.render_time(diff)})
                </p>
                <pre>{build.stdout}</pre>
                <pre>{build.stderr}</pre>
            </div>;
        } else {
            return <p key={index} onClick={this.toggle_open.bind(this, build.uuid)}>
                <b>{build.build}</b> (finished in {this.render_time(diff)})
            </p>;
        }
    }
    render_process(process: server.RunningProcess, index: number) {
        let is_open = this.state.open_uuids.some(u => u == process.uuid);
        if (is_open) {
            return <div key={index}>
                <p onClick={this.toggle_open.bind(this, process.uuid)}>
                    <b>{process.directory}</b> <a href="#" onClick={this.kill_process.bind(this, process.id)}>&times;</a>
                </p>
                <pre>{process.stdout}</pre>
                <pre>{process.stderr}</pre>
            </div>;
        } else {
            return <p key={index} onClick={this.toggle_open.bind(this, process.uuid)}>
                <b>{process.directory}</b> <a href="#" onClick={this.kill_process.bind(this, process.id)}>&times;</a>
            </p>;
        }
    }

    kill_process(id: number, ev: React.MouseEvent<HTMLAnchorElement>) {
        ev.preventDefault();
        ev.stopPropagation();

        fetch("/api/kill/" + id).then(r => r.text()).then(r => {
            if (r !== "Ok") {
                alert("Could not kill process\n" + r);
            }
        });

        return false;
    }
    render_project(project: server.Project, index: number) {
        return <p key={index}><b>{project.name}</b> {project.builds.map(this.render_build.bind(this, project))}</p>;
    }
    render_build(project: server.Project, build: server.Build, index: number) {
        return <button key={index} onClick={this.start_build.bind(this, project, build)}>{build.name}</button>;
    }
    toggle_open(uuid: string, ev: React.MouseEvent<HTMLElement>) {
        ev.preventDefault();
        ev.stopPropagation();

        let uuids = this.state.open_uuids;
        let index = uuids.findIndex(u => u == uuid);
        if (index !== -1) {
            uuids.splice(index, 1);
        } else {
            uuids.push(uuid);
        }
        this.setState({
            open_uuids: uuids,
        });

        return false;
    }
    start_build(project: server.Project, build: server.Build, ev: React.MouseEvent<HTMLButtonElement>) {
        ev.preventDefault();
        ev.stopPropagation();

        fetch("/api/build/start/" + project.name + "/" + build.name).then(r => r.text()).then(t => {
            if (t != "Ok") {
                alert("Could not start build\n" + t);
            }
        });

        return false;
    }
    render() {
        if (!this.state.state) return <></>;
        return <>
            <div style={{ display: 'flex', flexDirection: 'row' }}>
                <div style={{ width: '50%' }}>
                    <h2>Processes:</h2>
                    {this.state.state.running_processes.map(this.render_process.bind(this))}
                </div>
                <div style={{ width: '50%' }}>
                    {this.state.state.projects.map(this.render_project.bind(this))}
                </div>
            </div>
            <div style={{ display: 'flex', flexDirection: 'row' }}>
                <div style={{ width: '50%' }}>
                    <h2>Running:</h2>
                    {this.state.state.running_builds.map(this.render_running_build.bind(this))}
                </div>
                <div style={{ width: '50%' }}>
                    <h2>Finished:</h2>
                    {this.state.state.finished_builds.map(this.render_finished_build.bind(this))}
                </div>
            </div>
        </>;
    }
}