import { invoke as TAURI_INVOKE } from "@tauri-apps/api/tauri";
import * as TAURI_API_EVENT from "@tauri-apps/api/event";
import { type WebviewWindowHandle as __WebviewWindowHandle__ } from "@tauri-apps/api/window";
import useSWR, { type SWRConfiguration } from "swr";
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

type __Result__<T, E> =
  | { status: "ok"; data: T }
  | { status: "error"; error: E };

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
  useSWR: (
    args: T,
    config?: SWRConfiguration<O, E>
  ) => ReturnType<typeof useSWR<O, E>>;
  useSWRMutation: (
    config?: SWRMutationConfiguration<O, E>
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
    useSWR(args, config) {
      return useSWR(
        [key, ...args],
        ([_key, ..._args]) => TAURI_INVOKE(_key, ..._args),
        config
      );
    },
    useSWRMutation(config) {
      return useSWRMutation<O, E, string, T>(
        key,
        (_, { arg }) => {
          return TAURI_INVOKE(key, ...arg);
        },
        config
      );
    },
  };
}
