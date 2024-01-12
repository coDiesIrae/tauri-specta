import { appWindow } from "@tauri-apps/api/window";
import { events } from "./bindings";

events.emptyEvent.listen((e) => console.log(e));
events.emptyEvent(appWindow).listen((e) => console.log(e));
