function base64ToArrayBuffer(base64) {
    if (!base64) return undefined;
    var binary_string = window.atob(base64.replace(/_/g, '/').replace(/-/g, '+'));
    var len = binary_string.length;
    var bytes = new Uint8Array(len);
    for (var i = 0; i < len; i++) {
        bytes[i] = binary_string.charCodeAt(i);
    }
    return bytes.buffer;
}

function arrayBufferToBase64(buffer) {
    if (!buffer) return undefined;
    var binary = '';
    var bytes = new Uint8Array(buffer);
    for (var i = 0; i < bytes.byteLength; i++) {
        binary += String.fromCharCode(bytes[i]);
    }
    return window.btoa(binary).replace(/\//g, '_').replace(/\+/g, '-').replace(/=/g, '');
}

export async function createPasskeyBinding(options_json) {
    try {
        const options = JSON.parse(options_json);
        
        // Convert the challenge
        options.publicKey.challenge = base64ToArrayBuffer(options.publicKey.challenge);
        options.publicKey.user.id = base64ToArrayBuffer(options.publicKey.user.id);
        
        if (options.publicKey.excludeCredentials) {
            options.publicKey.excludeCredentials = options.publicKey.excludeCredentials.map(c => {
                c.id = base64ToArrayBuffer(c.id);
                return c;
            });
        }

        const cred = await navigator.credentials.create(options);
        
        return JSON.stringify({
            id: cred.id,
            rawId: arrayBufferToBase64(cred.rawId),
            type: cred.type,
            response: {
                clientDataJSON: arrayBufferToBase64(cred.response.clientDataJSON),
                attestationObject: arrayBufferToBase64(cred.response.attestationObject),
                authenticatorData: cred.response.authenticatorData ? arrayBufferToBase64(cred.response.authenticatorData) : undefined,
                publicKeyAlgorithm: cred.response.getPublicKeyAlgorithm ? cred.response.getPublicKeyAlgorithm() : undefined,
                publicKey: cred.response.getPublicKey ? arrayBufferToBase64(cred.response.getPublicKey()) : undefined,
                transports: cred.response.getTransports ? cred.response.getTransports() : [],
            }
        });
    } catch (e) {
        throw new Error(e.message || "Failed to create passkey");
    }
}

export async function getPasskeyBinding(options_json) {
    try {
        const options = JSON.parse(options_json);
        
        options.publicKey.challenge = base64ToArrayBuffer(options.publicKey.challenge);
        
        if (options.publicKey.allowCredentials) {
            options.publicKey.allowCredentials = options.publicKey.allowCredentials.map(c => {
                c.id = base64ToArrayBuffer(c.id);
                return c;
            });
        }

        const cred = await navigator.credentials.get(options);
        
        return JSON.stringify({
            id: cred.id,
            rawId: arrayBufferToBase64(cred.rawId),
            type: cred.type,
            response: {
                clientDataJSON: arrayBufferToBase64(cred.response.clientDataJSON),
                authenticatorData: arrayBufferToBase64(cred.response.authenticatorData),
                signature: arrayBufferToBase64(cred.response.signature),
                userHandle: cred.response.userHandle ? arrayBufferToBase64(cred.response.userHandle) : undefined,
            }
        });
    } catch (e) {
        throw new Error(e.message || "Failed to get passkey");
    }
}
