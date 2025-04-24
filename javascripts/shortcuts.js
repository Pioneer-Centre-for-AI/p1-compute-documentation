keyboard$
    .subscribe(key => {
        if (key.mode === "search")
            return;
        switch (key.type) {
            case "ArrowLeft":
                const prev = document.querySelector("link[rel=prev]");
                if (typeof prev !== "undefined")
                    location.href = prev.href;
                break;
            case "ArrowRight":
                const next = document.querySelector("link[rel=next]");
                if (typeof next !== "undefined")
                    location.href = next.href;
                break;
        }
    });
