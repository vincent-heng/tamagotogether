document.addEventListener('DOMContentLoaded', () => {
    const elements = {
        moodText: document.getElementById('mood-text'),
        feedsCount: document.getElementById('feeds-count'),
        feedMessage: document.getElementById('feed-message'),
        feedBtn: document.getElementById('feed-btn'),
        playSection: document.getElementById('play-section'),
        playfulnessText: document.getElementById('playfulness-text'),
        playsCount: document.getElementById('plays-count'),
        playMessage: document.getElementById('play-message'),
        playBtn: document.getElementById('play-btn'),
        foxPlaceholder: document.querySelector('.fox-placeholder')
    };

    const getApiPath = (endpoint) => {
        const basePath = window.location.pathname.replace(/\/$/, '');
        return `${basePath}/api/${endpoint}`;
    };

    const updateVisuals = (data) => {
        const fox = elements.foxPlaceholder;
        
        fox.className = 'fox-placeholder';
        
        let emoji = '🦊';
        if (data.level_id <= 3) {
            fox.classList.add('mood-sad');
            emoji = '😿';
        } else if (data.level_id <= 6) {
            fox.classList.add('mood-neutral');
        } else if (data.level_id < 10) {
            fox.classList.add('mood-happy');
            emoji = '😊🦊';
        } else {
            fox.classList.add('mood-radiant');
            emoji = '✨🦊✨';
        }
        
        let animClass = 'anim-floating';
        if (data.level_id === 10) {
            const playfulness = data.playfulness_level || 1;
            if (playfulness <= 4) {
                animClass = 'anim-breathing';
            } else if (playfulness <= 8) {
                animClass = 'anim-bouncing';
            } else {
                animClass = 'anim-hilarious';
            }
        }
        fox.classList.add(animClass);
        fox.textContent = emoji;
    };

    /**
     * Update the main UI with status data.
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

        // Play section: only visible when happiness is at max (level 10)
        if (data.level_id === 10) {
            elements.playSection.classList.remove('hidden');
            elements.foxPlaceholder.classList.add('rolling');
            elements.playfulnessText.textContent = data.playfulness_text;
            elements.playsCount.textContent = `A joué ${data.plays_today} fois aujourd'hui`;

            if (data.player_plays_today >= 3) {
                elements.playBtn.disabled = true;
                elements.playBtn.textContent = "Déjà joué";
            } else {
                elements.playBtn.disabled = false;
                elements.playBtn.textContent = "Jouer";
            }
        } else {
            elements.playSection.classList.add('hidden');
            elements.foxPlaceholder.classList.remove('rolling');
        }

        if (data.message) {
            elements.feedMessage.textContent = data.message;
        }
        
        updateVisuals(data);
    };

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

    const feedTamagotchi = async () => {
        if (elements.feedBtn.disabled) return;
        
        elements.feedBtn.disabled = true;
        elements.feedBtn.textContent = "Chargement...";

        try {
            const res = await fetch(getApiPath('feed'), { method: 'POST' });
            if (!res.ok) throw new Error(`HTTP error! status: ${res.status}`);
            const data = await res.json();
            updateUI(data);
            // Re-fetch full status to update play section if level changed to 10
            fetchStatus();
        } catch (error) {
            console.error("Error feeding:", error);
            elements.feedMessage.textContent = "Erreur lors de l'action";
            elements.feedBtn.disabled = false;
        }
    };

    const playWithTamagotchi = async () => {
        if (elements.playBtn.disabled) return;

        elements.playBtn.disabled = true;
        elements.playBtn.textContent = "Chargement...";

        try {
            const res = await fetch(getApiPath('play'), { method: 'POST' });
            if (!res.ok) throw new Error(`HTTP error! status: ${res.status}`);
            const data = await res.json();

            elements.playfulnessText.textContent = data.playfulness_text;
            elements.playsCount.textContent = `A joué ${data.plays_today} fois aujourd'hui`;
            elements.playMessage.textContent = data.message;

            if (data.player_plays_today >= 3) {
                elements.playBtn.disabled = true;
                elements.playBtn.textContent = "Déjà joué";
            } else {
                elements.playBtn.disabled = false;
                elements.playBtn.textContent = "Jouer";
            }
            
            updateVisuals(data);
        } catch (error) {
            console.error("Error playing:", error);
            elements.playMessage.textContent = "Erreur lors de l'action";
            elements.playBtn.disabled = false;
        }
    };

    elements.feedBtn.addEventListener('click', feedTamagotchi);
    elements.playBtn.addEventListener('click', playWithTamagotchi);
    
    fetchStatus();
});
