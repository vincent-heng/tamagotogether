document.addEventListener('DOMContentLoaded', () => {
    const elements = {
        moodText: document.getElementById('mood-text'),
        feedsCount: document.getElementById('feeds-count'),
        feedMessage: document.getElementById('feed-message'),
        feedBtn: document.getElementById('feed-btn')
    };

    /**
     * Get the correct API path, handling potential subpath deployment.
     */
    const getApiPath = (endpoint) => {
        const basePath = window.location.pathname.replace(/\/$/, '');
        return `${basePath}/api/${endpoint}`;
    };

    /**
     * Update the UI with fresh state data.
     */
    const updateUI = (data) => {
        elements.moodText.textContent = data.mood_text;
        elements.feedsCount.textContent = `Nourri ${data.feeds_today} fois aujourd'hui`;

        if (data.has_fed_today) {
            elements.feedBtn.disabled = true;
            elements.feedBtn.textContent = "Déjà nourri";
        } else {
            elements.feedBtn.disabled = false;
            elements.feedBtn.textContent = "Nourrir";
        }

        if (data.message) {
            elements.feedMessage.textContent = data.message;
        }
    };

    /**
     * Fetch current Tamagotchi status from the server.
     */
    const fetchStatus = async () => {
        try {
            const res = await fetch(getApiPath('state'));
            if (!res.ok) throw new Error(`HTTP error! status: ${res.status}`);
            const data = await res.json();
            updateUI(data);
        } catch (error) {
            console.error("Error fetching state:", error);
            elements.moodText.textContent = "Erreur de connexion";
        }
    };

    /**
     * Send a feed request to the server.
     */
    const feedTamagotchi = async () => {
        if (elements.feedBtn.disabled) return;
        
        elements.feedBtn.disabled = true;
        elements.feedBtn.textContent = "Chargement...";

        try {
            const res = await fetch(getApiPath('feed'), {
                method: 'POST'
            });
            if (!res.ok) throw new Error(`HTTP error! status: ${res.status}`);
            const data = await res.json();
            updateUI(data);
        } catch (error) {
            console.error("Error feeding:", error);
            elements.feedMessage.textContent = "Erreur lors de l'action";
            elements.feedBtn.disabled = false;
        }
    };

    elements.feedBtn.addEventListener('click', feedTamagotchi);
    
    // Initial fetch
    fetchStatus();
});
