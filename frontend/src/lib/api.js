import { get } from 'svelte/store';
import { effectiveDirectoryId, isProduction } from './stores/directoryStore';

// Determine if we're running in Docker or locally
const isDocker = window.location.port === '5001'; // Docker frontend port

// Select the appropriate API URL
const API_URL = import.meta.env.API_URL || 
                (isDocker ? "http://localhost:8001/api" : "http://localhost:8000/api");

console.log("API_URL:", API_URL);

// Add a function to get the auth token from localStorage
function getAuthToken() {
  return localStorage.getItem('authToken');
}

export async function fetchDirectories() {
  if (get(isProduction)) {
    const directoryId = get(effectiveDirectoryId);
    if (!directoryId) {
      throw new Error("No directory configured for production");
    }
    return [{ id: directoryId, name: "Production Directory" }];
  }

  const response = await fetch(`${API_URL}/directories`);
  if (!response.ok) {
    throw new Error("Failed to fetch directories");
  }
  return response.json();
}

export async function fetchListings() {
  const directoryId = get(effectiveDirectoryId);
  if (!directoryId) {
    throw new Error("No directory selected");
  }
  const response = await fetch(`${API_URL}/listings?directory_id=${directoryId}`);
  console.log("Response:", response);
  if (!response.ok) {
    throw new Error("Failed to fetch businesses");
  }
  return response.json();
}

export async function searchListings(query) {
  const directoryId = get(effectiveDirectoryId);
  if (!directoryId) {
    throw new Error("No directory selected");
  }
  const response = await fetch(`${API_URL}/listings/search?q=${query}&directory_id=${directoryId}`);
  console.log("Response:", response);
  if (!response.ok) {
    throw new Error("Failed to search listings");
  }
  return response.json();
}

export async function fetchListingById(id) {
  const response = await fetch(`http://localhost:8000/api/listing/${id}`, {
    credentials: 'include',
  });
  
  console.log('Response:', response);
  console.log('Response headers:', response.headers);
  
  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`);
  }
  
  const text = await response.text();
  
  let data;
  try {
    data = JSON.parse(text);
  } catch (e) {
    console.error('Error parsing JSON:', e);
    throw new Error('Invalid JSON in response');
  }
  return data;
}

export async function loginUser(credentials) {
  console.log("Logging in user");
  const response = await fetch(`${API_URL}/login`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Accept': 'application/json',
    },
    body: JSON.stringify(credentials),
    credentials: 'include',
  });
  if (!response.ok) {
    const error = new Error('Failed to login');
    error.status = response.status;
    throw error;
  }
  return response.json();
}

export async function registerUser(userData) {
  console.log("Registering user");
  const response = await fetch(`${API_URL}/register`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(userData),
  });
  if (!response.ok) {
    const error = new Error('Failed to register user');
    error.status = response.status;
    throw error;
  }
  return response.json();
}