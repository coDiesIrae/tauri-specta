// This file was generated by [tauri-specta](https://github.com/oscartbeaumont/tauri-specta). Do not edit this file manually.

export const commands = {
  hello_world: command<[myName: string], string, never>(
    "plugin:tauri-specta|hello_world"
  ),
  goodbye_world: command<[], string, never>(
    "plugin:tauri-specta|goodbye_world"
  ),
  has_error: command<[], string, number>("plugin:tauri-specta|has_error"),
  result_struct: command<[], MyStruct, null>(
    "plugin:tauri-specta|result_struct"
  ),
  my_struct: command<[], MyStruct, never>("plugin:tauri-specta|my_struct"),
  generic_struct: command<[], { some_field: string }, null>(
    "plugin:tauri-specta|generic_struct"
  ),
  generic: command<[], null, never>("plugin:tauri-specta|generic"),
};

export const events = __makeEvents__<{
  demoEvent: DemoEvent;
  emptyEvent: EmptyEvent;
}>({
  demoEvent: "plugin:tauri-specta:demo-event",
  emptyEvent: "plugin:tauri-specta:empty-event",
});

const {} = commands.hello_world.useSWR();

/** user-defined types **/

export type DemoEvent = string;
export type EmptyEvent = null;
export type GenericStruct<T> = { some_field: T };
export type MyStruct = { some_field: string };

/** tauri-specta globals **/

import { invoke as TAURI_INVOKE } from "@tauri-apps/api/tauri";
import * as TAURI_API_EVENT from "@tauri-apps/api/event";
import { type WebviewWindowHandle as __WebviewWindowHandle__ } from "@tauri-apps/api/window";
import useSWR, { Fetcher, type SWRConfiguration } from "swr";
import useSWRMutation, { SWRMutationConfiguration } from "swr/mutation";

type __EventObj__<T> = {
  listen: (
    cb: TAURI_API_EVENT.EventCallback<T>
  ) => ReturnType<typeof TAURI_API_EVENT.listen<T>>;
  once: (
    cb: TAURI_API_EVENT.EventCallback<T>
  ) => ReturnType<typeof TAURI_API_EVENT.once<T>>;
  emit: T extends null
    ? (payload?: T) => ReturnType<typeof TAURI_API_EVENT.emit>
    : (payload: T) => ReturnType<typeof TAURI_API_EVENT.emit>;
};

function __makeEvents__<T extends Record<string, any>>(
  mappings: Record<keyof T, string>
) {
  return new Proxy(
    {} as unknown as {
      [K in keyof T]: __EventObj__<T[K]> & {
        (handle: __WebviewWindowHandle__): __EventObj__<T[K]>;
      };
    },
    {
      get: (_, event) => {
        const name = mappings[event as keyof T];

        return new Proxy((() => {}) as any, {
          apply: (_, __, [window]: [__WebviewWindowHandle__]) => ({
            listen: (arg: any) => window.listen(name, arg),
            once: (arg: any) => window.once(name, arg),
            emit: (arg: any) => window.emit(name, arg),
          }),
          get: (_, command: keyof __EventObj__<any>) => {
            switch (command) {
              case "listen":
                return (arg: any) => TAURI_API_EVENT.listen(name, arg);
              case "once":
                return (arg: any) => TAURI_API_EVENT.once(name, arg);
              case "emit":
                return (arg: any) => TAURI_API_EVENT.emit(name, arg);
            }
          },
        });
      },
    }
  );
}

type __CommandType<T extends any[], O, E> = {
  async(...args: T): Promise<O>;
  key: string;
  useSWR: <
    Options extends SWRConfiguration<O, E, Fetcher<O, string>> | undefined =
      | SWRConfiguration<O, E, Fetcher<O, string>>
      | undefined
  >(
    args?: T,
    config?: Options
  ) => ReturnType<typeof useSWR<O, E, Options>>;
  useSWRMutation: <
    MutationOptions extends
      | SWRMutationConfiguration<O, E, string, T>
      | undefined = SWRMutationConfiguration<O, E, string, T> | undefined
  >(
    config?: MutationOptions
  ) => ReturnType<typeof useSWRMutation<O, E, string, T>>;
};

function command<T extends any[], O = any, E = any>(
  key: string
): __CommandType<T, O, E> {
  return {
    async(...args: T) {
      return TAURI_INVOKE(key, ...args);
    },
    key,
    useSWR(args = [] as unknown as T, config) {
      return useSWR(
        [key, ...args],
        ([_key, ..._args]) => TAURI_INVOKE(_key, ..._args),
        config
      );
    },
    useSWRMutation(config) {
      return useSWRMutation(
        key,
        (_key, { arg: _args }) => {
          return TAURI_INVOKE(_key, ..._args);
        },
        config
      );
    },
  };
}
