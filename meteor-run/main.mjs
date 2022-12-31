import got from 'got'

while (true) {

    const pl0 = await got.get('http://localhost:3000/api/0/publication/rundownPlaylists/{}/aa').json()
    const pl = pl0[0]

    if (!pl) throw new Error('No playlist!')
    if (!pl.nextPartInstanceId) throw new Error('no next part instance id')
    

    await got.post(`http://localhost:3000/api/0/action/take/aa/0/${pl._id}/${pl.currentPartInstanceId}`)

    console.log(pl)


    await new Promise(resolve => setTimeout(resolve, 2000))
}