export function bufferToBase64url(buffer) {
    const bytes = new Uint8Array(buffer);
    const len = bytes.byteLength;
    let str = '';
    for (let i = 0; i < len; i++) {
        str += String.fromCharCode(bytes[i]);
    }
    return window.btoa(str)
        .replace(/\+/g, '-')
        .replace(/\//g, '_')
        .replace(/=/g, '');
}

export function base64urlToBuffer(base64url) {
    let b64 = base64url.replace(/-/g, '+').replace(/_/g, '/');
    while (b64.length % 4 !== 0) {
        b64 += '=';
    }
    const binStr = window.atob(b64);
    const len = binStr.length;
    const bytes = new Uint8Array(len);
    for (let i = 0; i < len; i++) {
        bytes[i] = binStr.charCodeAt(i);
    }
    return bytes.buffer;
}

export async function registerDevice(optionsJson) {
    const options = JSON.parse(optionsJson);
    const pk = options.publicKey;
    
    // Transform webauthn-rs Base64Url payloads into hardware ArrayBuffers
    pk.challenge = base64urlToBuffer(pk.challenge);
    pk.user.id = base64urlToBuffer(pk.user.id);
    if (pk.excludeCredentials) {
        for (let cred of pk.excludeCredentials) {
            cred.id = base64urlToBuffer(cred.id);
        }
    }

    const credential = await navigator.credentials.create({ publicKey: pk });

    return JSON.stringify({
        id: credential.id,
        rawId: bufferToBase64url(credential.rawId),
        response: {
            clientDataJSON: bufferToBase64url(credential.response.clientDataJSON),
            attestationObject: bufferToBase64url(credential.response.attestationObject)
        },
        type: credential.type
    });
}

export async function authenticateDevice(optionsJson) {
    const options = JSON.parse(optionsJson);
    const pk = options.publicKey;

    // Transform webauthn-rs Base64Url payloads into hardware ArrayBuffers
    pk.challenge = base64urlToBuffer(pk.challenge);
    if (pk.allowCredentials) {
        for (let cred of pk.allowCredentials) {
            cred.id = base64urlToBuffer(cred.id);
        }
    }

    const credential = await navigator.credentials.get({ publicKey: pk });

    return JSON.stringify({
        id: credential.id,
        rawId: bufferToBase64url(credential.rawId),
        type: credential.type,
        response: {
            authenticatorData: bufferToBase64url(credential.response.authenticatorData),
            clientDataJSON: bufferToBase64url(credential.response.clientDataJSON),
            signature: bufferToBase64url(credential.response.signature),
            userHandle: credential.response.userHandle ? bufferToBase64url(credential.response.userHandle) : null
        }
    });
}
