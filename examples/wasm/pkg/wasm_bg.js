let wasm;
export function __wbg_set_wasm(val) {
    wasm = val;
}


const heap = new Array(128).fill(undefined);

heap.push(undefined, null, true, false);

function getObject(idx) { return heap[idx]; }

let heap_next = heap.length;

function dropObject(idx) {
    if (idx < 132) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

const lTextDecoder = typeof TextDecoder === 'undefined' ? (0, module.require)('util').TextDecoder : TextDecoder;

let cachedTextDecoder = new lTextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

let cachedUint8Memory0 = null;

function getUint8Memory0() {
    if (cachedUint8Memory0 === null || cachedUint8Memory0.byteLength === 0) {
        cachedUint8Memory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8Memory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}

let WASM_VECTOR_LEN = 0;

const lTextEncoder = typeof TextEncoder === 'undefined' ? (0, module.require)('util').TextEncoder : TextEncoder;

let cachedTextEncoder = new lTextEncoder('utf-8');

const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
}
    : function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
        read: arg.length,
        written: buf.length
    };
});

function passStringToWasm0(arg, malloc, realloc) {

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8Memory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8Memory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8Memory0().subarray(ptr + offset, ptr + len);
        const ret = encodeString(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

let cachedInt32Memory0 = null;

function getInt32Memory0() {
    if (cachedInt32Memory0 === null || cachedInt32Memory0.byteLength === 0) {
        cachedInt32Memory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachedInt32Memory0;
}

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(state => {
    wasm.__wbindgen_export_2.get(state.dtor)(state.a, state.b)
});

function makeMutClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {
        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            if (--state.cnt === 0) {
                wasm.__wbindgen_export_2.get(state.dtor)(a, state.b);
                CLOSURE_DTORS.unregister(state);
            } else {
                state.a = a;
            }
        }
    };
    real.original = state;
    CLOSURE_DTORS.register(real, state, state);
    return real;
}
function __wbg_adapter_26(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h5ebc7de778013349(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_29(arg0, arg1) {
    wasm._dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h4fb3131a885e2d10(arg0, arg1);
}

function __wbg_adapter_32(arg0, arg1, arg2, arg3) {
    wasm._dyn_core__ops__function__FnMut__A_B___Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h730177e083439bb7(arg0, arg1, addHeapObject(arg2), addHeapObject(arg3));
}

function __wbg_adapter_43(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h980e48f90fd823fa(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_48(arg0, arg1) {
    wasm._dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__ha4a7bd3a440c1fab(arg0, arg1);
}

function __wbg_adapter_51(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h5f09a98a8848d92b(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_64(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__hf21ab624e941c700(arg0, arg1, addHeapObject(arg2));
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8Memory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

let cachedUint32Memory0 = null;

function getUint32Memory0() {
    if (cachedUint32Memory0 === null || cachedUint32Memory0.byteLength === 0) {
        cachedUint32Memory0 = new Uint32Array(wasm.memory.buffer);
    }
    return cachedUint32Memory0;
}

function getArrayU32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint32Memory0().subarray(ptr / 4, ptr / 4 + len);
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_exn_store(addHeapObject(e));
    }
}
function __wbg_adapter_600(arg0, arg1, arg2, arg3) {
    wasm.wasm_bindgen__convert__closures__invoke2_mut__ha68be6068ceadf9b(arg0, arg1, addHeapObject(arg2), addHeapObject(arg3));
}

const MiniGPUWebFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_minigpuweb_free(ptr >>> 0));
/**
*/
export class MiniGPUWeb {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(MiniGPUWeb.prototype);
        obj.__wbg_ptr = ptr;
        MiniGPUWebFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        MiniGPUWebFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_minigpuweb_free(ptr);
    }
    /**
    */
    constructor() {
        const ret = wasm.minigpuweb_new();
        return takeObject(ret);
    }
    /**
    * @returns {Promise<void>}
    */
    loop_render() {
        const ret = wasm.minigpuweb_loop_render(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
    * @param {string} key
    * @param {Uint8Array} value
    */
    update_obj_map(key, value) {
        const ptr0 = passStringToWasm0(key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArray8ToWasm0(value, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        wasm.minigpuweb_update_obj_map(this.__wbg_ptr, ptr0, len0, ptr1, len1);
    }
}

export function __wbindgen_object_drop_ref(arg0) {
    takeObject(arg0);
};

export function __wbindgen_cb_drop(arg0) {
    const obj = takeObject(arg0).original;
    if (obj.cnt-- == 1) {
        obj.a = 0;
        return true;
    }
    const ret = false;
    return ret;
};

export function __wbindgen_object_clone_ref(arg0) {
    const ret = getObject(arg0);
    return addHeapObject(ret);
};

export function __wbg_minigpuweb_new(arg0) {
    const ret = MiniGPUWeb.__wrap(arg0);
    return addHeapObject(ret);
};

export function __wbindgen_is_undefined(arg0) {
    const ret = getObject(arg0) === undefined;
    return ret;
};

export function __wbindgen_string_new(arg0, arg1) {
    const ret = getStringFromWasm0(arg0, arg1);
    return addHeapObject(ret);
};

export function __wbg_new_abda76e883ba8a5f() {
    const ret = new Error();
    return addHeapObject(ret);
};

export function __wbg_stack_658279fe44541cf6(arg0, arg1) {
    const ret = getObject(arg1).stack;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};

export function __wbg_error_f851667af71bcfc6(arg0, arg1) {
    let deferred0_0;
    let deferred0_1;
    try {
        deferred0_0 = arg0;
        deferred0_1 = arg1;
        console.error(getStringFromWasm0(arg0, arg1));
    } finally {
        wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
    }
};

export function __wbg_instanceof_GpuValidationError_776dc042f9752ecb(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof GPUValidationError;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};

export function __wbg_message_e73620d927b54373(arg0, arg1) {
    const ret = getObject(arg1).message;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};

export function __wbg_instanceof_GpuOutOfMemoryError_3621d9e8ec05691e(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof GPUOutOfMemoryError;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};

export function __wbg_error_c4453561fa6c2209(arg0) {
    const ret = getObject(arg0).error;
    return addHeapObject(ret);
};

export function __wbg_instanceof_GpuDeviceLostInfo_22f963b61044b3b1(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof GPUDeviceLostInfo;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};

export function __wbg_reason_3af8e4afbe0efdd8(arg0) {
    const ret = getObject(arg0).reason;
    return addHeapObject(ret);
};

export function __wbg_message_3bef8c43f84eab9c(arg0, arg1) {
    const ret = getObject(arg1).message;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};

export function __wbindgen_string_get(arg0, arg1) {
    const obj = getObject(arg1);
    const ret = typeof(obj) === 'string' ? obj : undefined;
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};

export function __wbg_has_1509b2ce6759dc2a(arg0, arg1, arg2) {
    const ret = getObject(arg0).has(getStringFromWasm0(arg1, arg2));
    return ret;
};

export function __wbg_maxTextureDimension1D_ea59b0f0cc2e29cd(arg0) {
    const ret = getObject(arg0).maxTextureDimension1D;
    return ret;
};

export function __wbg_maxTextureDimension2D_00984ba245729ced(arg0) {
    const ret = getObject(arg0).maxTextureDimension2D;
    return ret;
};

export function __wbg_maxTextureDimension3D_95c3d3adb6d66ec5(arg0) {
    const ret = getObject(arg0).maxTextureDimension3D;
    return ret;
};

export function __wbg_maxTextureArrayLayers_68f4a1218a54fa93(arg0) {
    const ret = getObject(arg0).maxTextureArrayLayers;
    return ret;
};

export function __wbg_maxBindGroups_e76fb8650a4459d7(arg0) {
    const ret = getObject(arg0).maxBindGroups;
    return ret;
};

export function __wbg_maxBindingsPerBindGroup_2af20f39aef3fd86(arg0) {
    const ret = getObject(arg0).maxBindingsPerBindGroup;
    return ret;
};

export function __wbg_maxDynamicUniformBuffersPerPipelineLayout_074c891075b375b7(arg0) {
    const ret = getObject(arg0).maxDynamicUniformBuffersPerPipelineLayout;
    return ret;
};

export function __wbg_maxDynamicStorageBuffersPerPipelineLayout_b91e3e6efb7b7a8c(arg0) {
    const ret = getObject(arg0).maxDynamicStorageBuffersPerPipelineLayout;
    return ret;
};

export function __wbg_maxSampledTexturesPerShaderStage_76354979d03a2b27(arg0) {
    const ret = getObject(arg0).maxSampledTexturesPerShaderStage;
    return ret;
};

export function __wbg_maxSamplersPerShaderStage_fe8d223de90e5459(arg0) {
    const ret = getObject(arg0).maxSamplersPerShaderStage;
    return ret;
};

export function __wbg_maxStorageBuffersPerShaderStage_bced69629145d26d(arg0) {
    const ret = getObject(arg0).maxStorageBuffersPerShaderStage;
    return ret;
};

export function __wbg_maxStorageTexturesPerShaderStage_fcf51f22620c0092(arg0) {
    const ret = getObject(arg0).maxStorageTexturesPerShaderStage;
    return ret;
};

export function __wbg_maxUniformBuffersPerShaderStage_b3b013238400f0c0(arg0) {
    const ret = getObject(arg0).maxUniformBuffersPerShaderStage;
    return ret;
};

export function __wbg_maxUniformBufferBindingSize_194fd7147cf2e95a(arg0) {
    const ret = getObject(arg0).maxUniformBufferBindingSize;
    return ret;
};

export function __wbg_maxStorageBufferBindingSize_78504383af63ac53(arg0) {
    const ret = getObject(arg0).maxStorageBufferBindingSize;
    return ret;
};

export function __wbg_maxVertexBuffers_78c71ff19beac74b(arg0) {
    const ret = getObject(arg0).maxVertexBuffers;
    return ret;
};

export function __wbg_maxBufferSize_0c7ed57407582d40(arg0) {
    const ret = getObject(arg0).maxBufferSize;
    return ret;
};

export function __wbg_maxVertexAttributes_c11cb018a9c5a224(arg0) {
    const ret = getObject(arg0).maxVertexAttributes;
    return ret;
};

export function __wbg_maxVertexBufferArrayStride_c53560cc036cb477(arg0) {
    const ret = getObject(arg0).maxVertexBufferArrayStride;
    return ret;
};

export function __wbg_minUniformBufferOffsetAlignment_4880e6786cb7ec5d(arg0) {
    const ret = getObject(arg0).minUniformBufferOffsetAlignment;
    return ret;
};

export function __wbg_minStorageBufferOffsetAlignment_9913f200aee2c749(arg0) {
    const ret = getObject(arg0).minStorageBufferOffsetAlignment;
    return ret;
};

export function __wbg_maxInterStageShaderComponents_f9243ac86242eb18(arg0) {
    const ret = getObject(arg0).maxInterStageShaderComponents;
    return ret;
};

export function __wbg_maxColorAttachments_d33b1d22c06a6fc5(arg0) {
    const ret = getObject(arg0).maxColorAttachments;
    return ret;
};

export function __wbg_maxColorAttachmentBytesPerSample_637fd3ac394c14ee(arg0) {
    const ret = getObject(arg0).maxColorAttachmentBytesPerSample;
    return ret;
};

export function __wbg_maxComputeWorkgroupStorageSize_7e5bc378e5a62367(arg0) {
    const ret = getObject(arg0).maxComputeWorkgroupStorageSize;
    return ret;
};

export function __wbg_maxComputeInvocationsPerWorkgroup_1ed5b24d52720f8a(arg0) {
    const ret = getObject(arg0).maxComputeInvocationsPerWorkgroup;
    return ret;
};

export function __wbg_maxComputeWorkgroupSizeX_56b713fb17f8c261(arg0) {
    const ret = getObject(arg0).maxComputeWorkgroupSizeX;
    return ret;
};

export function __wbg_maxComputeWorkgroupSizeY_13040bdf12fd4e65(arg0) {
    const ret = getObject(arg0).maxComputeWorkgroupSizeY;
    return ret;
};

export function __wbg_maxComputeWorkgroupSizeZ_8c8594730967472d(arg0) {
    const ret = getObject(arg0).maxComputeWorkgroupSizeZ;
    return ret;
};

export function __wbg_maxComputeWorkgroupsPerDimension_4094c8501eea36ce(arg0) {
    const ret = getObject(arg0).maxComputeWorkgroupsPerDimension;
    return ret;
};

export function __wbg_instanceof_GpuAdapter_32bc80c8c30adaa0(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof GPUAdapter;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};

export function __wbg_queue_2bddd1700cb0bec2(arg0) {
    const ret = getObject(arg0).queue;
    return addHeapObject(ret);
};

export function __wbindgen_is_object(arg0) {
    const val = getObject(arg0);
    const ret = typeof(val) === 'object' && val !== null;
    return ret;
};

export function __wbg_instanceof_GpuCanvasContext_b3bff0de75efe6fd(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof GPUCanvasContext;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};

export function __wbg_getMappedRange_1216b00d6d7803de(arg0, arg1, arg2) {
    const ret = getObject(arg0).getMappedRange(arg1, arg2);
    return addHeapObject(ret);
};

export function __wbg_Window_94d759f1f207a15b(arg0) {
    const ret = getObject(arg0).Window;
    return addHeapObject(ret);
};

export function __wbg_WorkerGlobalScope_b13c8cef62388de9(arg0) {
    const ret = getObject(arg0).WorkerGlobalScope;
    return addHeapObject(ret);
};

export function __wbg_gpu_1f3675e2d4aa88f4(arg0) {
    const ret = getObject(arg0).gpu;
    return addHeapObject(ret);
};

export function __wbg_requestAdapter_e6f12701c7a38391(arg0, arg1) {
    const ret = getObject(arg0).requestAdapter(getObject(arg1));
    return addHeapObject(ret);
};

export function __wbindgen_number_new(arg0) {
    const ret = arg0;
    return addHeapObject(ret);
};

export function __wbg_requestDevice_727ad8687b0d6553(arg0, arg1) {
    const ret = getObject(arg0).requestDevice(getObject(arg1));
    return addHeapObject(ret);
};

export function __wbg_features_b56ebab8f515839e(arg0) {
    const ret = getObject(arg0).features;
    return addHeapObject(ret);
};

export function __wbg_limits_be2f592b5e154a3d(arg0) {
    const ret = getObject(arg0).limits;
    return addHeapObject(ret);
};

export function __wbg_getPreferredCanvasFormat_012ef9f3b0238ffa(arg0) {
    const ret = getObject(arg0).getPreferredCanvasFormat();
    return addHeapObject(ret);
};

export function __wbg_configure_6cde48f0c99a3497(arg0, arg1) {
    getObject(arg0).configure(getObject(arg1));
};

export function __wbg_getCurrentTexture_95b5b88416fdb0c2(arg0) {
    const ret = getObject(arg0).getCurrentTexture();
    return addHeapObject(ret);
};

export function __wbg_features_4991b2a28904a253(arg0) {
    const ret = getObject(arg0).features;
    return addHeapObject(ret);
};

export function __wbg_limits_1aa8a49e0a8442cc(arg0) {
    const ret = getObject(arg0).limits;
    return addHeapObject(ret);
};

export function __wbg_createShaderModule_036b780a18124d9e(arg0, arg1) {
    const ret = getObject(arg0).createShaderModule(getObject(arg1));
    return addHeapObject(ret);
};

export function __wbg_createBindGroupLayout_313b4151e718ff1f(arg0, arg1) {
    const ret = getObject(arg0).createBindGroupLayout(getObject(arg1));
    return addHeapObject(ret);
};

export function __wbg_createBindGroup_2d6778f92445c8bf(arg0, arg1) {
    const ret = getObject(arg0).createBindGroup(getObject(arg1));
    return addHeapObject(ret);
};

export function __wbg_createPipelineLayout_9134c6c32c505ec8(arg0, arg1) {
    const ret = getObject(arg0).createPipelineLayout(getObject(arg1));
    return addHeapObject(ret);
};

export function __wbg_createRenderPipeline_2bfc852ce09914fc(arg0, arg1) {
    const ret = getObject(arg0).createRenderPipeline(getObject(arg1));
    return addHeapObject(ret);
};

export function __wbg_createComputePipeline_02674342979c6288(arg0, arg1) {
    const ret = getObject(arg0).createComputePipeline(getObject(arg1));
    return addHeapObject(ret);
};

export function __wbg_createBuffer_65c2fc555c46aa07(arg0, arg1) {
    const ret = getObject(arg0).createBuffer(getObject(arg1));
    return addHeapObject(ret);
};

export function __wbg_createTexture_5adbcf0db3fd41b4(arg0, arg1) {
    const ret = getObject(arg0).createTexture(getObject(arg1));
    return addHeapObject(ret);
};

export function __wbg_createSampler_942022241ecf4277(arg0, arg1) {
    const ret = getObject(arg0).createSampler(getObject(arg1));
    return addHeapObject(ret);
};

export function __wbg_createQuerySet_424dbf8130140914(arg0, arg1) {
    const ret = getObject(arg0).createQuerySet(getObject(arg1));
    return addHeapObject(ret);
};

export function __wbg_createCommandEncoder_1db1770ea9eab9af(arg0, arg1) {
    const ret = getObject(arg0).createCommandEncoder(getObject(arg1));
    return addHeapObject(ret);
};

export function __wbg_createRenderBundleEncoder_32896e68340fabc6(arg0, arg1) {
    const ret = getObject(arg0).createRenderBundleEncoder(getObject(arg1));
    return addHeapObject(ret);
};

export function __wbg_destroy_4f7ed2bbb4742899(arg0) {
    getObject(arg0).destroy();
};

export function __wbg_lost_42410660a8cd8819(arg0) {
    const ret = getObject(arg0).lost;
    return addHeapObject(ret);
};

export function __wbg_setonuncapturederror_4e4946a65c61f3ef(arg0, arg1) {
    getObject(arg0).onuncapturederror = getObject(arg1);
};

export function __wbg_pushErrorScope_a09c8b037ab27e15(arg0, arg1) {
    getObject(arg0).pushErrorScope(takeObject(arg1));
};

export function __wbg_popErrorScope_f8f0d4b6d5c635f9(arg0) {
    const ret = getObject(arg0).popErrorScope();
    return addHeapObject(ret);
};

export function __wbg_mapAsync_3b0a03a892fb22b3(arg0, arg1, arg2, arg3) {
    const ret = getObject(arg0).mapAsync(arg1 >>> 0, arg2, arg3);
    return addHeapObject(ret);
};

export function __wbg_unmap_7a0dddee82ac6ed3(arg0) {
    getObject(arg0).unmap();
};

export function __wbg_createView_0ab0576f1665c9ad(arg0, arg1) {
    const ret = getObject(arg0).createView(getObject(arg1));
    return addHeapObject(ret);
};

export function __wbg_destroy_199808599201ee27(arg0) {
    getObject(arg0).destroy();
};

export function __wbg_destroy_57694ff5aabbf32d(arg0) {
    getObject(arg0).destroy();
};

export function __wbg_getBindGroupLayout_a0d36a72bd39bb04(arg0, arg1) {
    const ret = getObject(arg0).getBindGroupLayout(arg1 >>> 0);
    return addHeapObject(ret);
};

export function __wbg_getBindGroupLayout_abc654a192f85d5e(arg0, arg1) {
    const ret = getObject(arg0).getBindGroupLayout(arg1 >>> 0);
    return addHeapObject(ret);
};

export function __wbg_copyBufferToBuffer_667953bc6dccb6b4(arg0, arg1, arg2, arg3, arg4, arg5) {
    getObject(arg0).copyBufferToBuffer(getObject(arg1), arg2, getObject(arg3), arg4, arg5);
};

export function __wbg_copyBufferToTexture_ca5b298687bed60a(arg0, arg1, arg2, arg3) {
    getObject(arg0).copyBufferToTexture(getObject(arg1), getObject(arg2), getObject(arg3));
};

export function __wbg_copyTextureToBuffer_cdf8118386295eb4(arg0, arg1, arg2, arg3) {
    getObject(arg0).copyTextureToBuffer(getObject(arg1), getObject(arg2), getObject(arg3));
};

export function __wbg_copyTextureToTexture_67678f03fd20bd23(arg0, arg1, arg2, arg3) {
    getObject(arg0).copyTextureToTexture(getObject(arg1), getObject(arg2), getObject(arg3));
};

export function __wbg_beginComputePass_a148b983810f6795(arg0, arg1) {
    const ret = getObject(arg0).beginComputePass(getObject(arg1));
    return addHeapObject(ret);
};

export function __wbg_end_28d311f5d435aa6d(arg0) {
    getObject(arg0).end();
};

export function __wbg_beginRenderPass_0b83360fd99b5810(arg0, arg1) {
    const ret = getObject(arg0).beginRenderPass(getObject(arg1));
    return addHeapObject(ret);
};

export function __wbg_end_e3cea1776c95d64f(arg0) {
    getObject(arg0).end();
};

export function __wbg_label_175c4f59b3eca611(arg0, arg1) {
    const ret = getObject(arg1).label;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};

export function __wbg_finish_d1d9eb9915c96a79(arg0, arg1) {
    const ret = getObject(arg0).finish(getObject(arg1));
    return addHeapObject(ret);
};

export function __wbg_finish_ce7d5c15fce975aa(arg0) {
    const ret = getObject(arg0).finish();
    return addHeapObject(ret);
};

export function __wbg_clearBuffer_2cc723ab6b818737(arg0, arg1, arg2) {
    getObject(arg0).clearBuffer(getObject(arg1), arg2);
};

export function __wbg_clearBuffer_78a94a2eda97eb5a(arg0, arg1, arg2, arg3) {
    getObject(arg0).clearBuffer(getObject(arg1), arg2, arg3);
};

export function __wbg_resolveQuerySet_22e31015a36a09d5(arg0, arg1, arg2, arg3, arg4, arg5) {
    getObject(arg0).resolveQuerySet(getObject(arg1), arg2 >>> 0, arg3 >>> 0, getObject(arg4), arg5 >>> 0);
};

export function __wbg_finish_2115db9e679c5aae(arg0) {
    const ret = getObject(arg0).finish();
    return addHeapObject(ret);
};

export function __wbg_finish_4a754149a60eddc0(arg0, arg1) {
    const ret = getObject(arg0).finish(getObject(arg1));
    return addHeapObject(ret);
};

export function __wbg_writeBuffer_4245ce84e6d772c9(arg0, arg1, arg2, arg3, arg4, arg5) {
    getObject(arg0).writeBuffer(getObject(arg1), arg2, getObject(arg3), arg4, arg5);
};

export function __wbg_usage_5e9a3548afbc3ebb(arg0) {
    const ret = getObject(arg0).usage;
    return ret;
};

export function __wbg_size_fc880d60ff425a47(arg0) {
    const ret = getObject(arg0).size;
    return ret;
};

export function __wbg_writeTexture_686a8160c3c5ddbb(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).writeTexture(getObject(arg1), getObject(arg2), getObject(arg3), getObject(arg4));
};

export function __wbg_copyExternalImageToTexture_87bdcc3260c6efba(arg0, arg1, arg2, arg3) {
    getObject(arg0).copyExternalImageToTexture(getObject(arg1), getObject(arg2), getObject(arg3));
};

export function __wbg_setPipeline_8630b264a9c4ec4b(arg0, arg1) {
    getObject(arg0).setPipeline(getObject(arg1));
};

export function __wbg_setBindGroup_17e73587d3c1be08(arg0, arg1, arg2) {
    getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2));
};

export function __wbg_setBindGroup_5a450a0e97199c15(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
    getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2), getArrayU32FromWasm0(arg3, arg4), arg5, arg6 >>> 0);
};

export function __wbg_dispatchWorkgroups_4bc133944e89d5e0(arg0, arg1, arg2, arg3) {
    getObject(arg0).dispatchWorkgroups(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0);
};

export function __wbg_dispatchWorkgroupsIndirect_8050acb60dd74a34(arg0, arg1, arg2) {
    getObject(arg0).dispatchWorkgroupsIndirect(getObject(arg1), arg2);
};

export function __wbg_setPipeline_a95b89d99620ba34(arg0, arg1) {
    getObject(arg0).setPipeline(getObject(arg1));
};

export function __wbg_setBindGroup_58e27d4cd266f187(arg0, arg1, arg2) {
    getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2));
};

export function __wbg_setBindGroup_f70bb0d0a5ace56d(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
    getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2), getArrayU32FromWasm0(arg3, arg4), arg5, arg6 >>> 0);
};

export function __wbg_setIndexBuffer_747e1ba3f58d7227(arg0, arg1, arg2, arg3) {
    getObject(arg0).setIndexBuffer(getObject(arg1), takeObject(arg2), arg3);
};

export function __wbg_setIndexBuffer_3f1635c89f72d661(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).setIndexBuffer(getObject(arg1), takeObject(arg2), arg3, arg4);
};

export function __wbg_setVertexBuffer_94a88edbfb4b07f8(arg0, arg1, arg2, arg3) {
    getObject(arg0).setVertexBuffer(arg1 >>> 0, getObject(arg2), arg3);
};

export function __wbg_setVertexBuffer_407067a9522118df(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).setVertexBuffer(arg1 >>> 0, getObject(arg2), arg3, arg4);
};

export function __wbg_draw_60508d893ce4e012(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).draw(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
};

export function __wbg_drawIndexed_d5c5dff02437a4f0(arg0, arg1, arg2, arg3, arg4, arg5) {
    getObject(arg0).drawIndexed(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4, arg5 >>> 0);
};

export function __wbg_drawIndirect_54f93ae4ccc85358(arg0, arg1, arg2) {
    getObject(arg0).drawIndirect(getObject(arg1), arg2);
};

export function __wbg_drawIndexedIndirect_bf668464170261b3(arg0, arg1, arg2) {
    getObject(arg0).drawIndexedIndirect(getObject(arg1), arg2);
};

export function __wbg_setPipeline_d7c9c55035f118a6(arg0, arg1) {
    getObject(arg0).setPipeline(getObject(arg1));
};

export function __wbg_setBindGroup_c6ab2e9583489b58(arg0, arg1, arg2) {
    getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2));
};

export function __wbg_setBindGroup_0bf976b9657f99bd(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
    getObject(arg0).setBindGroup(arg1 >>> 0, getObject(arg2), getArrayU32FromWasm0(arg3, arg4), arg5, arg6 >>> 0);
};

export function __wbg_setIndexBuffer_ea39707d8842fe03(arg0, arg1, arg2, arg3) {
    getObject(arg0).setIndexBuffer(getObject(arg1), takeObject(arg2), arg3);
};

export function __wbg_setIndexBuffer_04ba4ea48c8f80be(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).setIndexBuffer(getObject(arg1), takeObject(arg2), arg3, arg4);
};

export function __wbg_setVertexBuffer_907c60acf6dca161(arg0, arg1, arg2, arg3) {
    getObject(arg0).setVertexBuffer(arg1 >>> 0, getObject(arg2), arg3);
};

export function __wbg_setVertexBuffer_9a336bb112a33317(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).setVertexBuffer(arg1 >>> 0, getObject(arg2), arg3, arg4);
};

export function __wbg_draw_540a514f996a5d0d(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).draw(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
};

export function __wbg_drawIndexed_f717a07602ee2d18(arg0, arg1, arg2, arg3, arg4, arg5) {
    getObject(arg0).drawIndexed(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4, arg5 >>> 0);
};

export function __wbg_drawIndirect_c588ff54fb149aee(arg0, arg1, arg2) {
    getObject(arg0).drawIndirect(getObject(arg1), arg2);
};

export function __wbg_drawIndexedIndirect_bb5585ec7f45d269(arg0, arg1, arg2) {
    getObject(arg0).drawIndexedIndirect(getObject(arg1), arg2);
};

export function __wbg_setBlendConstant_496a0b5cc772c236(arg0, arg1) {
    getObject(arg0).setBlendConstant(getObject(arg1));
};

export function __wbg_setScissorRect_9b7e673d03036c37(arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).setScissorRect(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
};

export function __wbg_setViewport_85d18ceefd5180eb(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
    getObject(arg0).setViewport(arg1, arg2, arg3, arg4, arg5, arg6);
};

export function __wbg_setStencilReference_b4b1f7e586967a4d(arg0, arg1) {
    getObject(arg0).setStencilReference(arg1 >>> 0);
};

export function __wbg_executeBundles_16985086317c358a(arg0, arg1) {
    getObject(arg0).executeBundles(getObject(arg1));
};

export function __wbg_submit_afbd82b0d5056194(arg0, arg1) {
    getObject(arg0).submit(getObject(arg1));
};

export function __wbg_offsetX_d08eda91526f22a2(arg0) {
    const ret = getObject(arg0).offsetX;
    return ret;
};

export function __wbg_offsetY_3c895bb1534dfbf4(arg0) {
    const ret = getObject(arg0).offsetY;
    return ret;
};

export function __wbg_scheduler_8082c844a9cfc0df(arg0) {
    const ret = getObject(arg0).scheduler;
    return addHeapObject(ret);
};

export function __wbg_onpointerrawupdate_e087759b4021ec00(arg0) {
    const ret = getObject(arg0).onpointerrawupdate;
    return addHeapObject(ret);
};

export function __wbg_getCoalescedEvents_4665669d237be577(arg0) {
    const ret = getObject(arg0).getCoalescedEvents;
    return addHeapObject(ret);
};

export function __wbg_scheduler_6932606c19435996(arg0) {
    const ret = getObject(arg0).scheduler;
    return addHeapObject(ret);
};

export function __wbg_requestIdleCallback_081ddac93612a53e(arg0) {
    const ret = getObject(arg0).requestIdleCallback;
    return addHeapObject(ret);
};

export function __wbg_postTask_4674878f9a603824(arg0, arg1, arg2) {
    const ret = getObject(arg0).postTask(getObject(arg1), getObject(arg2));
    return addHeapObject(ret);
};

export function __wbg_requestFullscreen_a851d70cb190396a(arg0) {
    const ret = getObject(arg0).requestFullscreen;
    return addHeapObject(ret);
};

export function __wbg_requestFullscreen_f4349fb8a7429cf9(arg0) {
    const ret = getObject(arg0).requestFullscreen();
    return addHeapObject(ret);
};

export function __wbg_Window_cc0273a5da2c36dc(arg0) {
    const ret = getObject(arg0).Window;
    return addHeapObject(ret);
};

export function __wbg_webkitFullscreenElement_533c5f32e2ac8d0c(arg0) {
    const ret = getObject(arg0).webkitFullscreenElement;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_prototype_8e5075a5dd95f801() {
    const ret = ResizeObserverEntry.prototype;
    return addHeapObject(ret);
};

export function __wbg_webkitRequestFullscreen_8abcfecec7127495(arg0) {
    getObject(arg0).webkitRequestFullscreen();
};

export function __wbg_queueMicrotask_3cbae2ec6b6cd3d6(arg0) {
    const ret = getObject(arg0).queueMicrotask;
    return addHeapObject(ret);
};

export function __wbindgen_is_function(arg0) {
    const ret = typeof(getObject(arg0)) === 'function';
    return ret;
};

export function __wbg_queueMicrotask_481971b0d87f3dd4(arg0) {
    queueMicrotask(getObject(arg0));
};

export function __wbg_instanceof_Window_f401953a2cf86220(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof Window;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};

export function __wbg_document_5100775d18896c16(arg0) {
    const ret = getObject(arg0).document;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_navigator_6c8fa55c5cc8796e(arg0) {
    const ret = getObject(arg0).navigator;
    return addHeapObject(ret);
};

export function __wbg_devicePixelRatio_efc553b59506f64c(arg0) {
    const ret = getObject(arg0).devicePixelRatio;
    return ret;
};

export function __wbg_cancelIdleCallback_3a36cf77475b492b(arg0, arg1) {
    getObject(arg0).cancelIdleCallback(arg1 >>> 0);
};

export function __wbg_getComputedStyle_078292ffe423aded() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).getComputedStyle(getObject(arg1));
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
}, arguments) };

export function __wbg_matchMedia_66bb21e3ef19270c() { return handleError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).matchMedia(getStringFromWasm0(arg1, arg2));
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
}, arguments) };

export function __wbg_requestIdleCallback_cee8e1d6bdcfae9e() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).requestIdleCallback(getObject(arg1));
    return ret;
}, arguments) };

export function __wbg_cancelAnimationFrame_111532f326e480af() { return handleError(function (arg0, arg1) {
    getObject(arg0).cancelAnimationFrame(arg1);
}, arguments) };

export function __wbg_requestAnimationFrame_549258cfa66011f0() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).requestAnimationFrame(getObject(arg1));
    return ret;
}, arguments) };

export function __wbg_clearTimeout_ba63ae54a36e111e(arg0, arg1) {
    getObject(arg0).clearTimeout(arg1);
};

export function __wbg_setTimeout_d2b9a986d10a6182() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).setTimeout(getObject(arg1));
    return ret;
}, arguments) };

export function __wbg_setTimeout_c172d5704ef82276() { return handleError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).setTimeout(getObject(arg1), arg2);
    return ret;
}, arguments) };

export function __wbg_body_edb1908d3ceff3a1(arg0) {
    const ret = getObject(arg0).body;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_visibilityState_990071edf70b1c55(arg0) {
    const ret = getObject(arg0).visibilityState;
    return addHeapObject(ret);
};

export function __wbg_activeElement_fa7feca08f5028c0(arg0) {
    const ret = getObject(arg0).activeElement;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_fullscreenElement_1bef71098bd8dfde(arg0) {
    const ret = getObject(arg0).fullscreenElement;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_createElement_8bae7856a4bb7411() { return handleError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).createElement(getStringFromWasm0(arg1, arg2));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_getElementById_c369ff43f0db99cf(arg0, arg1, arg2) {
    const ret = getObject(arg0).getElementById(getStringFromWasm0(arg1, arg2));
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_querySelectorAll_4e0fcdb64cda2cd5() { return handleError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).querySelectorAll(getStringFromWasm0(arg1, arg2));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_setAttribute_3c9f6c303b696daa() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).setAttribute(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
}, arguments) };

export function __wbg_setPointerCapture_0fdaad7a916c8486() { return handleError(function (arg0, arg1) {
    getObject(arg0).setPointerCapture(arg1);
}, arguments) };

export function __wbg_style_c3fc3dd146182a2d(arg0) {
    const ret = getObject(arg0).style;
    return addHeapObject(ret);
};

export function __wbg_focus_39d4b8ba8ff9df14() { return handleError(function (arg0) {
    getObject(arg0).focus();
}, arguments) };

export function __wbg_navigator_56803b85352a0575(arg0) {
    const ret = getObject(arg0).navigator;
    return addHeapObject(ret);
};

export function __wbg_pointerId_e030fa156647fedd(arg0) {
    const ret = getObject(arg0).pointerId;
    return ret;
};

export function __wbg_pressure_99cd07399f942a7c(arg0) {
    const ret = getObject(arg0).pressure;
    return ret;
};

export function __wbg_pointerType_0f2f0383406aa7fa(arg0, arg1) {
    const ret = getObject(arg1).pointerType;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};

export function __wbg_getCoalescedEvents_14b443b6f75837a2(arg0) {
    const ret = getObject(arg0).getCoalescedEvents();
    return addHeapObject(ret);
};

export function __wbg_appendChild_580ccb11a660db68() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).appendChild(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_contains_fdfd1dc667f36695(arg0, arg1) {
    const ret = getObject(arg0).contains(getObject(arg1));
    return ret;
};

export function __wbg_get_8cd5eba00ab6304f(arg0, arg1) {
    const ret = getObject(arg0)[arg1 >>> 0];
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
};

export function __wbg_inlineSize_ff0e40258cefeba2(arg0) {
    const ret = getObject(arg0).inlineSize;
    return ret;
};

export function __wbg_blockSize_73f4e5608c08713d(arg0) {
    const ret = getObject(arg0).blockSize;
    return ret;
};

export function __wbg_instanceof_HtmlCanvasElement_46bdbf323b0b18d1(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof HTMLCanvasElement;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};

export function __wbg_setwidth_080107476e633963(arg0, arg1) {
    getObject(arg0).width = arg1 >>> 0;
};

export function __wbg_setheight_dc240617639f1f51(arg0, arg1) {
    getObject(arg0).height = arg1 >>> 0;
};

export function __wbg_getContext_df50fa48a8876636() { return handleError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).getContext(getStringFromWasm0(arg1, arg2));
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
}, arguments) };

export function __wbg_new_4e95a9abecc83cd4() { return handleError(function (arg0) {
    const ret = new IntersectionObserver(getObject(arg0));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_disconnect_e694940ce6d0ef91(arg0) {
    getObject(arg0).disconnect();
};

export function __wbg_observe_538a6d1df0deb993(arg0, arg1) {
    getObject(arg0).observe(getObject(arg1));
};

export function __wbg_setwidth_83d936c4b04dcbec(arg0, arg1) {
    getObject(arg0).width = arg1 >>> 0;
};

export function __wbg_setheight_6025ba0d58e6cc8c(arg0, arg1) {
    getObject(arg0).height = arg1 >>> 0;
};

export function __wbg_getContext_c102f659d540d068() { return handleError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).getContext(getStringFromWasm0(arg1, arg2));
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
}, arguments) };

export function __wbg_deltaX_206576827ededbe5(arg0) {
    const ret = getObject(arg0).deltaX;
    return ret;
};

export function __wbg_deltaY_032e327e216f2b2b(arg0) {
    const ret = getObject(arg0).deltaY;
    return ret;
};

export function __wbg_deltaMode_294b2eaf54047265(arg0) {
    const ret = getObject(arg0).deltaMode;
    return ret;
};

export function __wbg_signal_a61f78a3478fd9bc(arg0) {
    const ret = getObject(arg0).signal;
    return addHeapObject(ret);
};

export function __wbg_new_0d76b0581eca6298() { return handleError(function () {
    const ret = new AbortController();
    return addHeapObject(ret);
}, arguments) };

export function __wbg_abort_2aa7521d5690750e(arg0) {
    getObject(arg0).abort();
};

export function __wbg_addEventListener_53b787075bd5e003() { return handleError(function (arg0, arg1, arg2, arg3) {
    getObject(arg0).addEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
}, arguments) };

export function __wbg_removeEventListener_92cb9b3943463338() { return handleError(function (arg0, arg1, arg2, arg3) {
    getObject(arg0).removeEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
}, arguments) };

export function __wbg_ctrlKey_008695ce60a588f5(arg0) {
    const ret = getObject(arg0).ctrlKey;
    return ret;
};

export function __wbg_shiftKey_1e76dbfcdd36a4b4(arg0) {
    const ret = getObject(arg0).shiftKey;
    return ret;
};

export function __wbg_altKey_07da841b54bd3ed6(arg0) {
    const ret = getObject(arg0).altKey;
    return ret;
};

export function __wbg_metaKey_86bfd3b0d3a8083f(arg0) {
    const ret = getObject(arg0).metaKey;
    return ret;
};

export function __wbg_button_367cdc7303e3cf9b(arg0) {
    const ret = getObject(arg0).button;
    return ret;
};

export function __wbg_buttons_d004fa75ac704227(arg0) {
    const ret = getObject(arg0).buttons;
    return ret;
};

export function __wbg_movementX_b800a0cacd14d9bf(arg0) {
    const ret = getObject(arg0).movementX;
    return ret;
};

export function __wbg_movementY_7907e03eb8c0ea1e(arg0) {
    const ret = getObject(arg0).movementY;
    return ret;
};

export function __wbg_isIntersecting_082397a1d66e2e35(arg0) {
    const ret = getObject(arg0).isIntersecting;
    return ret;
};

export function __wbg_debug_5fb96680aecf5dc8(arg0) {
    console.debug(getObject(arg0));
};

export function __wbg_error_8e3928cfb8a43e2b(arg0) {
    console.error(getObject(arg0));
};

export function __wbg_error_6e987ee48d9fdf45(arg0, arg1) {
    console.error(getObject(arg0), getObject(arg1));
};

export function __wbg_info_530a29cb2e4e3304(arg0) {
    console.info(getObject(arg0));
};

export function __wbg_log_5bb5f88f245d7762(arg0) {
    console.log(getObject(arg0));
};

export function __wbg_warn_63bbae1730aead09(arg0) {
    console.warn(getObject(arg0));
};

export function __wbg_media_bcef0e2ec4383569(arg0, arg1) {
    const ret = getObject(arg1).media;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};

export function __wbg_matches_e14ed9ff8291cf24(arg0) {
    const ret = getObject(arg0).matches;
    return ret;
};

export function __wbg_addListener_143ad0a501fabc3a() { return handleError(function (arg0, arg1) {
    getObject(arg0).addListener(getObject(arg1));
}, arguments) };

export function __wbg_removeListener_46f3ee00c5b95320() { return handleError(function (arg0, arg1) {
    getObject(arg0).removeListener(getObject(arg1));
}, arguments) };

export function __wbg_getPropertyValue_fa32ee1811f224cb() { return handleError(function (arg0, arg1, arg2, arg3) {
    const ret = getObject(arg1).getPropertyValue(getStringFromWasm0(arg2, arg3));
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
}, arguments) };

export function __wbg_removeProperty_fa6d48e2923dcfac() { return handleError(function (arg0, arg1, arg2, arg3) {
    const ret = getObject(arg1).removeProperty(getStringFromWasm0(arg2, arg3));
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
}, arguments) };

export function __wbg_setProperty_ea7d15a2b591aa97() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
    getObject(arg0).setProperty(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
}, arguments) };

export function __wbg_setonmessage_93bdba94dcd46c04(arg0, arg1) {
    getObject(arg0).onmessage = getObject(arg1);
};

export function __wbg_close_a5883ed21dc3d115(arg0) {
    getObject(arg0).close();
};

export function __wbg_postMessage_fbddfe9314af804e() { return handleError(function (arg0, arg1) {
    getObject(arg0).postMessage(getObject(arg1));
}, arguments) };

export function __wbg_start_5a293222bc398f51(arg0) {
    getObject(arg0).start();
};

export function __wbg_persisted_cbb7e3c657029516(arg0) {
    const ret = getObject(arg0).persisted;
    return ret;
};

export function __wbg_altKey_2e6c34c37088d8b1(arg0) {
    const ret = getObject(arg0).altKey;
    return ret;
};

export function __wbg_ctrlKey_bb5b6fef87339703(arg0) {
    const ret = getObject(arg0).ctrlKey;
    return ret;
};

export function __wbg_shiftKey_5911baf439ab232b(arg0) {
    const ret = getObject(arg0).shiftKey;
    return ret;
};

export function __wbg_metaKey_6bf4ae4e83a11278(arg0) {
    const ret = getObject(arg0).metaKey;
    return ret;
};

export function __wbg_location_f7b033ddfc516739(arg0) {
    const ret = getObject(arg0).location;
    return ret;
};

export function __wbg_repeat_f64b916c6eed0685(arg0) {
    const ret = getObject(arg0).repeat;
    return ret;
};

export function __wbg_key_dccf9e8aa1315a8e(arg0, arg1) {
    const ret = getObject(arg1).key;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};

export function __wbg_code_3b0c3912a2351163(arg0, arg1) {
    const ret = getObject(arg1).code;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};

export function __wbg_port1_d51a1bd2c33125d0(arg0) {
    const ret = getObject(arg0).port1;
    return addHeapObject(ret);
};

export function __wbg_port2_f522a81e92362e7e(arg0) {
    const ret = getObject(arg0).port2;
    return addHeapObject(ret);
};

export function __wbg_new_34615e164dc78975() { return handleError(function () {
    const ret = new MessageChannel();
    return addHeapObject(ret);
}, arguments) };

export function __wbg_preventDefault_b1a4aafc79409429(arg0) {
    getObject(arg0).preventDefault();
};

export function __wbg_new_61d4f20a1c08a45c() { return handleError(function (arg0) {
    const ret = new ResizeObserver(getObject(arg0));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_disconnect_6675f32e2ae8deb7(arg0) {
    getObject(arg0).disconnect();
};

export function __wbg_observe_a79646ce7bb08cb8(arg0, arg1) {
    getObject(arg0).observe(getObject(arg1));
};

export function __wbg_observe_dc0ebcd59ee7cd17(arg0, arg1, arg2) {
    getObject(arg0).observe(getObject(arg1), getObject(arg2));
};

export function __wbg_unobserve_55c93518cad6ac06(arg0, arg1) {
    getObject(arg0).unobserve(getObject(arg1));
};

export function __wbg_width_1e8430024cb82aba(arg0) {
    const ret = getObject(arg0).width;
    return ret;
};

export function __wbg_height_0c1394f089d7bb71(arg0) {
    const ret = getObject(arg0).height;
    return ret;
};

export function __wbg_contentRect_bce644376332c7a5(arg0) {
    const ret = getObject(arg0).contentRect;
    return addHeapObject(ret);
};

export function __wbg_devicePixelContentBoxSize_d5bcdcd5e96671f3(arg0) {
    const ret = getObject(arg0).devicePixelContentBoxSize;
    return addHeapObject(ret);
};

export function __wbg_performance_eeefc685c9bc38b4(arg0) {
    const ret = getObject(arg0).performance;
    return addHeapObject(ret);
};

export function __wbg_now_e0d8ec93dd25766a(arg0) {
    const ret = getObject(arg0).now();
    return ret;
};

export function __wbg_get_bd8e338fbd5f5cc8(arg0, arg1) {
    const ret = getObject(arg0)[arg1 >>> 0];
    return addHeapObject(ret);
};

export function __wbg_length_cd7af8117672b8b8(arg0) {
    const ret = getObject(arg0).length;
    return ret;
};

export function __wbg_new_16b304a2cfa7ff4a() {
    const ret = new Array();
    return addHeapObject(ret);
};

export function __wbg_newnoargs_e258087cd0daa0ea(arg0, arg1) {
    const ret = new Function(getStringFromWasm0(arg0, arg1));
    return addHeapObject(ret);
};

export function __wbg_call_27c0f87801dedf93() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).call(getObject(arg1));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_new_72fb9a18b5ae2624() {
    const ret = new Object();
    return addHeapObject(ret);
};

export function __wbg_self_ce0dbfc45cf2f5be() { return handleError(function () {
    const ret = self.self;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_window_c6fb939a7f436783() { return handleError(function () {
    const ret = window.window;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_globalThis_d1e6af4856ba331b() { return handleError(function () {
    const ret = globalThis.globalThis;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_global_207b558942527489() { return handleError(function () {
    const ret = global.global;
    return addHeapObject(ret);
}, arguments) };

export function __wbg_push_a5b05aedc7234f9f(arg0, arg1) {
    const ret = getObject(arg0).push(getObject(arg1));
    return ret;
};

export function __wbg_call_b3ca7c6051f9bec1() { return handleError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).call(getObject(arg1), getObject(arg2));
    return addHeapObject(ret);
}, arguments) };

export function __wbg_instanceof_Object_71ca3c0a59266746(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof Object;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};

export function __wbg_getOwnPropertyDescriptor_fcb32c9a1f90b136(arg0, arg1) {
    const ret = Object.getOwnPropertyDescriptor(getObject(arg0), getObject(arg1));
    return addHeapObject(ret);
};

export function __wbg_is_010fdc0f4ab96916(arg0, arg1) {
    const ret = Object.is(getObject(arg0), getObject(arg1));
    return ret;
};

export function __wbg_valueOf_a0b7c836f68a054b(arg0) {
    const ret = getObject(arg0).valueOf();
    return addHeapObject(ret);
};

export function __wbg_new_81740750da40724f(arg0, arg1) {
    try {
        var state0 = {a: arg0, b: arg1};
        var cb0 = (arg0, arg1) => {
            const a = state0.a;
            state0.a = 0;
            try {
                return __wbg_adapter_600(a, state0.b, arg0, arg1);
            } finally {
                state0.a = a;
            }
        };
        const ret = new Promise(cb0);
        return addHeapObject(ret);
    } finally {
        state0.a = state0.b = 0;
    }
};

export function __wbg_resolve_b0083a7967828ec8(arg0) {
    const ret = Promise.resolve(getObject(arg0));
    return addHeapObject(ret);
};

export function __wbg_catch_0260e338d10f79ae(arg0, arg1) {
    const ret = getObject(arg0).catch(getObject(arg1));
    return addHeapObject(ret);
};

export function __wbg_then_0c86a60e8fcfe9f6(arg0, arg1) {
    const ret = getObject(arg0).then(getObject(arg1));
    return addHeapObject(ret);
};

export function __wbg_then_a73caa9a87991566(arg0, arg1, arg2) {
    const ret = getObject(arg0).then(getObject(arg1), getObject(arg2));
    return addHeapObject(ret);
};

export function __wbg_buffer_12d079cc21e14bdb(arg0) {
    const ret = getObject(arg0).buffer;
    return addHeapObject(ret);
};

export function __wbg_newwithbyteoffsetandlength_aa4a17c33a06e5cb(arg0, arg1, arg2) {
    const ret = new Uint8Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
    return addHeapObject(ret);
};

export function __wbg_new_63b92bc8671ed464(arg0) {
    const ret = new Uint8Array(getObject(arg0));
    return addHeapObject(ret);
};

export function __wbg_set_a47bac70306a19a7(arg0, arg1, arg2) {
    getObject(arg0).set(getObject(arg1), arg2 >>> 0);
};

export function __wbg_length_c20a40f15020d68a(arg0) {
    const ret = getObject(arg0).length;
    return ret;
};

export function __wbg_buffer_dd7f74bc60f1faab(arg0) {
    const ret = getObject(arg0).buffer;
    return addHeapObject(ret);
};

export function __wbg_set_1f9b04f170055d33() { return handleError(function (arg0, arg1, arg2) {
    const ret = Reflect.set(getObject(arg0), getObject(arg1), getObject(arg2));
    return ret;
}, arguments) };

export function __wbindgen_debug_string(arg0, arg1) {
    const ret = debugString(getObject(arg1));
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getInt32Memory0()[arg0 / 4 + 1] = len1;
    getInt32Memory0()[arg0 / 4 + 0] = ptr1;
};

export function __wbindgen_throw(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
};

export function __wbindgen_memory() {
    const ret = wasm.memory;
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper303(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 9, __wbg_adapter_26);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper304(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 9, __wbg_adapter_29);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper305(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 9, __wbg_adapter_32);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper306(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 9, __wbg_adapter_26);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper307(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 9, __wbg_adapter_26);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper308(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 9, __wbg_adapter_26);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper309(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 9, __wbg_adapter_26);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper2733(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 1196, __wbg_adapter_43);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper2735(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 1196, __wbg_adapter_43);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper3143(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 1244, __wbg_adapter_48);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper3144(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 1244, __wbg_adapter_51);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper3145(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 1244, __wbg_adapter_51);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper3146(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 1244, __wbg_adapter_51);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper3147(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 1244, __wbg_adapter_51);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper3148(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 1244, __wbg_adapter_51);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper3149(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 1244, __wbg_adapter_51);
    return addHeapObject(ret);
};

export function __wbindgen_closure_wrapper3231(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 1309, __wbg_adapter_64);
    return addHeapObject(ret);
};

