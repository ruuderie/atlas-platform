import { get } from 'svelte/store';
import { effectiveDirectoryId, isProduction } from './stores/directoryStore';
import { env } from './stores/authStore';
import { loadUser } from './stores/userStore';
import { login, logout } from './auth';
import { setUser, clearUser } from './stores/userStore';
import { browser } from '$app/environment';

// Initialize api with empty objects as default
let api = {
  user: {},
  listing: {},
  admin: {}
};

// Determine if we're running in a browser or in a container
const isBrowser = typeof window !== 'undefined';

// Use different URLs based on environment
const API_URL = isBrowser 
  ? (import.meta.env.VITE_BROWSER_API_URL || 'http://admin.rustsveltebusinessdirectory.orb.local:8000')
  : (import.meta.env.VITE_CONTAINER_API_URL || 'http://localhost:8000');

console.log("Using API_URL:", API_URL);

if (browser) {
  async function refreshToken() {
    try {
      console.log("Refreshing token");
      const response = await fetch(`${API_URL}/refresh-token`, {
        method: 'POST',
        headers: getAuthHeaders(),
      });

      if (!response.ok) {
        console.error("Failed to refresh token. Status:", response.status);
        return { success: false, error: `Failed to refresh token. Status: ${response.status}` };
      } else {
        console.log("Token refreshed successfully");
      }

      const data = await response.json();
      console.log('Refresh token response data:', data);

      localStorage.setItem('authToken', data.token);
      localStorage.setItem('refreshToken', data.refresh_token);
      console.log('New auth token and refresh token set in localStorage');

      return { success: true, token: data.token, refreshToken: data.refresh_token };
    } catch (error) {
      console.error('Error in refreshToken:', error);
      return { success: false, error: error.message };
    }
  }

  function getAuthHeaders() {
    const token = localStorage.getItem('authToken');
    return {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json',
    };
  }

  async function apiCall(endpoint, options = {}, isPublic = false) {
    console.log("API call endpoint:", endpoint);
    
    // Ensure endpoint starts with a slash if it doesn't already
    const normalizedEndpoint = endpoint.startsWith('/') ? endpoint : `/${endpoint}`;
    
    // Use API_URL directly from environment variable
    const fullUrl = `${API_URL}${normalizedEndpoint}`;
    
    console.log("Final API URL:", fullUrl);
    
    if (!isPublic) {
      options.headers = { ...options.headers, ...getAuthHeaders() };
    }

    try {
      console.log("Fetch options:", JSON.stringify(options));
      const response = await fetch(fullUrl, options);
      console.log("Response status:", response.status);
      
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      return response.json();
    } catch (error) {
      console.error('API call failed:', error);
      throw error;
    }
  }

  async function verifySession() {
    try {
      console.log("Verifying session");
      console.log("Auth headers:", getAuthHeaders());
      console.log("API_URL:", API_URL);
      const response = await fetch(`${API_URL}/validate-session`, {
        method: 'GET',
        headers: getAuthHeaders(),
      });
      console.log("Response:", response);

      if (!response.ok) {
        console.error("Failed to verify session. Status:", response.status);
        return { isValid: false, error: `Failed to verify session. Status: ${response.status}` };
      }

      const data = await response.json();
      
      // Store user data if it exists
      if (data.user) {
        localStorage.setItem('userData', JSON.stringify(data.user));
      }

      return { isValid: true, user: data.user };
    } catch (error) {
      console.error('Error in verifySession:', error);
      return { isValid: false, error: error.message };
    }
  }

  const userApi = {
    login: async (credentials) => {
      console.log("Attempting to log in user:", credentials.email);
      try {
        const response = await apiCall('/login', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(credentials),
        }, true);
        console.log('Login response:', response);
        login(response.token, response.refresh_token, response.user);
        loadUser();  // Add this line to load the user data after login
        return response;
      } catch (error) {
        console.error('Login failed:', error);
        throw error;
      }
    },
    register: (userData) => {
      console.log("Registering user");
      return apiCall('/register', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(userData),
      }, true);
    },
    logout: async () => {
      try {
        await apiCall('/logout', { method: 'POST' });
        localStorage.removeItem('authToken');
        localStorage.removeItem('refreshToken');
        clearUser();
        isAuthenticated.set(false);
      } catch (error) {
        console.error('Logout failed:', error);
        throw error;
      }
    },
    getProfile: () => apiCall('/users/profile'),
    updateProfile: (profileData) => apiCall('/users/profile', { 
      method: 'PUT', 
      body: JSON.stringify(profileData) 
    }),
    verifySession: verifySession,
    refreshToken: refreshToken,
  };

  const listingApi = {
    fetchListings: () => {
      const directoryId = get(effectiveDirectoryId);
      if (!directoryId) {
        throw new Error("No directory selected");
      }
      return apiCall(`/listings?directory_id=${directoryId}`, {}, true);
    },
    searchListings: (query) => {
      const directoryId = get(effectiveDirectoryId);
      if (!directoryId) {
        throw new Error("No directory selected");
      }
      return apiCall(`/listings/search?q=${query}&directory_id=${directoryId}`, {}, true);
    },
    fetchListingById: (id) => apiCall(`/listings/${id}`, {}, true),
  };

  const adminApi = {
    fetchDashboardStats: () => {
      if (env === 'production') {
        return apiCall('/admin/dashboard-stats');
      } else {
        console.log('Using fake dashboard stats for non-production environment');
        return new Promise(resolve => {
          setTimeout(() => {
            resolve({
              totalUsers: 150000,
              activeListings: 75000,
              adPurchases: 12000,
              revenue: 1800000,
              totalCategories: 500,
              monthlyRevenue: [500000, 750000, 800000, 1250000, 1400000, 1750000, 2050000],
              userGrowth: [60000, 80000, 94250, 101250, 115741, 135741, 168521],
              listingGrowth: [60000, 62500, 65000, 67500, 70000, 72500, 75000],
              adSalesGrowth: [9000, 9500, 10000, 10500, 11000, 11500, 12000]
            });
          }, 500);
        });
      }
    },
    fetchAdPurchases: () => apiCall('/admin/ad-purchases'),
    fetchUsers: () => apiCall('/admin/users'),
    fetchDirectories: () => apiCall('/admin/directories'),
    fetchUserById: (userId) => apiCall(`/admin/users/${userId}`),
    updateUser: (userId, userData) => apiCall(`/admin/users/${userId}`, {
      method: 'PUT',
      body: JSON.stringify(userData)
    }),
    fetchCustomers: (page = 1, itemsPerPage = 10) => 
      apiCall(`/admin/customers?page=${page}&items_per_page=${itemsPerPage}`),
    
    fetchCustomerById: (id) => 
      apiCall(`/admin/customers/${id}`),
    
    updateCustomer: (id, customerData) => 
      apiCall(`/admin/customers/${id}`, {
        method: 'PUT',
        body: JSON.stringify(customerData)
      }),
    
    createCustomer: (customerData) => 
      apiCall('/admin/customers', {
        method: 'POST',
        body: JSON.stringify(customerData)
      }),
    
    deleteCustomer: (id) => 
      apiCall(`/admin/customers/${id}`, {
        method: 'DELETE'
      }),
    
    resetCustomerPassword: (id) => 
      apiCall(`/admin/customers/${id}/reset-password`, {
        method: 'POST'
      }),
  };

  // Assign the API implementations when in browser environment
  api = {
    user: userApi,
    listing: listingApi,
    admin: adminApi,
  };
}

// Export the api object (will be populated in browser, empty on server)
export { api };
