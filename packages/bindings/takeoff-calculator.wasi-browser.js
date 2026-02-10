import {
	getDefaultContext as __emnapiGetDefaultContext,
	instantiateNapiModule as __emnapiInstantiateNapiModule,
	WASI as __WASI,
} from "@napi-rs/wasm-runtime";

const __wasi = new __WASI({
	version: "preview1",
});

const __wasmUrl = new URL(
	"./takeoff-calculator.wasm32-wasi.wasm",
	import.meta.url,
).href;
const __emnapiContext = __emnapiGetDefaultContext();

const __sharedMemory = new WebAssembly.Memory({
	initial: 4000,
	maximum: 65536,
	shared: true,
});

const __wasmFile = await fetch(__wasmUrl).then((res) => res.arrayBuffer());

const {
	instance: __napiInstance,
	module: __wasiModule,
	napiModule: __napiModule,
} = await __emnapiInstantiateNapiModule(__wasmFile, {
	context: __emnapiContext,
	asyncWorkPoolSize: 4,
	wasi: __wasi,
	onCreateWorker() {
		const worker = new Worker(
			new URL("./wasi-worker-browser.mjs", import.meta.url),
			{
				type: "module",
			},
		);

		return worker;
	},
	overwriteImports(importObject) {
		importObject.env = {
			...importObject.env,
			...importObject.napi,
			...importObject.emnapi,
			memory: __sharedMemory,
		};
		return importObject;
	},
	beforeInit({ instance }) {
		for (const name of Object.keys(instance.exports)) {
			if (name.startsWith("__napi_register__")) {
				instance.exports[name]();
			}
		}
	},
});
export default __napiModule.exports;
export const ContourWrapper = __napiModule.exports.ContourWrapper;
export const GroupWrapper = __napiModule.exports.GroupWrapper;
export const MeasurementWrapper = __napiModule.exports.MeasurementWrapper;
export const TakeoffStateHandler = __napiModule.exports.TakeoffStateHandler;
export const plus100 = __napiModule.exports.plus100;
export const plus200 = __napiModule.exports.plus200;
export const UnitValue = __napiModule.exports.UnitValue;
export const MeasurementType = __napiModule.exports.MeasurementType;
export const simplifyPolyline = __napiModule.exports.simplifyPolyline;
export const Unit = __napiModule.exports.Unit;
export const UnitValueItemType = __napiModule.exports.UnitValueItemType;
